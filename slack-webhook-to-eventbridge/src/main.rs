//! This Lambda is compatible with AWS WebAPI. Its supposed to be connected to the Slack Event Subscription and will send incoming Slack Events to EventBridge.
//!
//! # Setup
//!
//! * Create a lambda with the binary from this repository using runtime `provided.al2`
//! and anything as handler. (More Infos about paramters below).
//! * Add permissions for `secretsmanager:GetSecretValue` and `events:PutEvents` to the
//! lambdas iam role
//!
//! # Parameters
//!
//! The lambda function has a few parameters. You can define them via
//! environment variables.
//!
//! ## Environment Variables
//! ```sh
//! # AWS Secretsmanager Secret which contains the signing_secret in the following format:
//! # `{"signing_secret": ""}`
//! SLACK_SECRET=""
//! # Name of the AWS Eventbridge this lambda is supposed to send events to
//! EVENTBRIDGE_BUS_NAME=""
//! # Optional, skip if not required. off | error | warn | info (default) | debug | trace
//! # Defines the log level
//! LOG_LEVEL=""
//! ```

#![deny(clippy::all, clippy::nursery)]
#![deny(nonstandard_style, rust_2018_idioms, unused_crate_dependencies)]

mod event;
mod response;

#[cfg(feature = "test")]
const TEST_DATA: &str = include_str!("../test.json");

const SLACK_TIMESTAMP_LEEWAY: std::time::Duration = std::time::Duration::from_secs(5 * 60);
const SLACK_HEADER_TIMESTAMP: &str = "X-Slack-Request-Timestamp";
const SLACK_HEADER_SIGNATURE: &str = "X-Slack-Signature";

const ENV_VAR_LOG_LEVEL: &str = "LOG_LEVEL";
const ENV_VAR_SLACK_SECRET: &str = "SLACK_SECRET";
const ENV_VAR_EVENTBRIDGE_BUS_NAME: &str = "EVENTBRIDGE_BUS_NAME";

#[derive(serde::Deserialize)]
struct SlackSecret {
    signing_secret: String,
}

struct Shared {
    eventbridge: aws_sdk_eventbridge::Client,
    slack_secret: SlackSecret,
    bus_name: String,
}

struct Runner;

#[async_trait::async_trait]
impl<'a> lambda_runtime_types::Runner<'a, Shared, event::ApiGatewayEvent, response::Response>
    for Runner
{
    async fn run(
        shared: &'a Shared,
        event: lambda_runtime_types::LambdaEvent<'a, event::ApiGatewayEvent>,
    ) -> anyhow::Result<response::Response> {
        if signature_valid(&shared.slack_secret.signing_secret, &event.event).is_none() {
            return Ok(response::Response::simple(http::StatusCode::FORBIDDEN));
        }
        let payload_str = event.event.body();
        let payload: serde_json::Value = match serde_json::from_str(&payload_str) {
            Err(_) => return Ok(response::Response::simple(http::StatusCode::BAD_REQUEST)),
            Ok(v) => v,
        };
        let event_type: &str = payload
            .get("event")
            .and_then(|e| e.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");
        let res = shared
            .eventbridge
            .put_events()
            .entries(
                aws_sdk_eventbridge::model::PutEventsRequestEntry::builder()
                    .source(env!("CARGO_PKG_NAME"))
                    .detail_type(format!("slack_event:{}", event_type))
                    .detail(payload_str)
                    .event_bus_name(&shared.bus_name)
                    .build(),
            )
            .send()
            .await;
        if let Err(err) = res {
            log::error!("Failed while trying to send event to EventBridge: {}", err);
            return Ok(response::Response::simple(
                http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        Ok(response::Response::simple(http::StatusCode::OK))
    }

    async fn setup(_region: &'a str) -> anyhow::Result<Shared> {
        use anyhow::Context;
        use std::str::FromStr;

        let log_level = std::env::var(ENV_VAR_LOG_LEVEL);
        let log_level = log_level.as_ref().map(AsRef::as_ref).unwrap_or("info");
        let log_level = log::LevelFilter::from_str(log_level)
            .with_context(|| format!("Invalid log_level: {}", log_level))?;
        simple_logger::SimpleLogger::new()
            .with_level(log_level)
            .init()
            .expect("Unable to setup logging");

        let aws_config = aws_config::load_from_env().await;
        let eventbridge = aws_sdk_eventbridge::Client::new(&aws_config);
        let secretsmanager = aws_sdk_secretsmanager::Client::new(&aws_config);

        let slack_secret = std::env::var(ENV_VAR_SLACK_SECRET)
            .with_context(|| format!("Env Variable {} must be set", ENV_VAR_SLACK_SECRET))?;
        let slack_secret = secretsmanager
            .get_secret_value()
            .secret_id(slack_secret)
            .send()
            .await
            .context("Unable to retrieve slack secret")?;
        let slack_secret: SlackSecret =
            match (slack_secret.secret_binary, slack_secret.secret_string) {
                (Some(binary), _) => serde_json::from_slice(binary.as_ref()),
                (_, Some(string)) => serde_json::from_str(&string),
                _ => anyhow::bail!("Slack Secret contains neither binary nor string secret"),
            }
            .context("Slack secret does not adhere to the required structure")?;

        let bus_name = std::env::var(ENV_VAR_EVENTBRIDGE_BUS_NAME).with_context(|| {
            format!("Env Variable {} must be set", ENV_VAR_EVENTBRIDGE_BUS_NAME)
        })?;

        Ok(Shared {
            eventbridge,
            slack_secret,
            bus_name,
        })
    }
}

fn signature_valid(signing_secret: &str, event: &event::ApiGatewayEvent) -> Option<()> {
    use hmac::Mac;

    let body = event.body();
    let headers = event.headers();

    // Extract signature
    let signature = headers
        .get(SLACK_HEADER_SIGNATURE)
        .and_then(|h| h.to_str().ok())?;
    if signature.len() < 3 {
        return None;
    }
    let (prefix, signature) = signature.as_bytes().split_at(3);
    let signature = hex::decode(signature).ok()?;

    // Extract timestamp
    let timestamp_original: &str = headers
        .get(SLACK_HEADER_TIMESTAMP)
        .and_then(|h| h.to_str().ok())?;
    let timestamp: u64 = timestamp_original.parse().ok()?;
    let now_minus_leeway = (get_current_timestamp() - SLACK_TIMESTAMP_LEEWAY).as_secs();
    if timestamp < now_minus_leeway {
        return None;
    }

    // Create signature from timestamp and body
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(signing_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(b"v0:");
    mac.update(timestamp_original.as_bytes());
    mac.update(b":");
    mac.update(body.as_bytes());

    // Verify signature
    match prefix {
        b"v0=" => mac.verify_slice(&signature).ok(),
        _ => None,
    }
}

fn get_current_timestamp() -> std::time::Duration {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Current time is prior to unix epoch")
}

/// Entrypoint for the lambda
pub fn main() -> anyhow::Result<()> {
    #[cfg(not(feature = "test"))]
    {
        lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
    }
    #[cfg(feature = "test")]
    {
        lambda_runtime_types::exec_test::<_, _, Runner, _>(TEST_DATA)
    }
}

//! This tool rotates Google Service User Credentials stored in AWS Secret Manager
//!
//! Setup
//!
//! * Create a Google IAM Role which contains the following permissions:
//!   * `iam.serviceAccountKeys.create`
//!   * `iam.serviceAccountKeys.delete`
//! * Attach the role with the following condition to the service user whoms
//! credentials you want to rotate:
//!   * `resource.name == "projects/-/serviceAccounts/<service-user-unique-id>"`
//! * The rotation function requires that the complete credential.json from the
//! service user is available somewhere in the secret, either as string containing
//! the json or as json object.
//! * Create a lambda with the binary from this repository using runtime `provided.al2`
//! and anything as handler. (More Infos about paramters below)
//! * Attach the lambda as rotation lambda to the AWS Secret Manager Secret
//!
//! # Parameters
//!
//! The lambda function has a few optional parameters. You can define them via
//! environment variables.
//!
//! ## Environment Variables
//! ```sh
//! # Optional, skip if not required.
//! # Defines the keys to traverse to find the credential.json. Example:
//! # { "test": [<credential.json>] }
//! # requires : JSON_PATH="[\"test\", 0]"
//! # default: JSON_PATH="[]" which expectes the credential.json to be at the top of the secret
//! JSON_PATH="[]"
//! # Optional, skip if not required. off | error | warn | info (default) | debug | trace
//! # Defines the log level
//! LOG_LEVEL=""
//! ```
//!

#![warn(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    deprecated_in_future,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    indirect_structural_match,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_crate_level_docs,
    missing_debug_implementations,
    missing_docs,
    missing_doc_code_examples,
    non_ascii_idents,
    private_doc_tests,
    trivial_casts,
    trivial_numeric_casts,
    unaligned_references,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
#![warn(
    clippy::cargo,
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::perf,
    clippy::style
)]
#![allow(
    clippy::future_not_send,
    clippy::multiple_crate_versions,
    clippy::redundant_pub_crate,
    clippy::wildcard_dependencies
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod google;

#[cfg(feature = "test")]
const TEST_DATA: &str = include_str!("../test.json");

const ENV_VAR_JSON_PATH: &str = "JSON_PATH";
const ENV_VAR_LOG_LEVEL: &str = "LOG_LEVEL";

struct Runner;

#[async_trait::async_trait]
impl<'a> lambda_runtime_types::rotate::RotateRunner<'a, (), serde_json::Value> for Runner {
    async fn setup(_region: &'a str) -> anyhow::Result<()> {
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
        Ok(())
    }

    async fn create(
        _shared: &'a (),
        mut secret_cur: lambda_runtime_types::rotate::SecretContainer<serde_json::Value>,
        _smc: &lambda_runtime_types::rotate::Smc,
    ) -> anyhow::Result<lambda_runtime_types::rotate::SecretContainer<serde_json::Value>> {
        use anyhow::Context;

        let value = get_mut_json_entry(&mut secret_cur)?;
        let credential_json = std::mem::replace(value, serde_json::Value::Null);
        let iam = google::Iam::new(credential_json).await?;
        let new_key = iam.create_key().await?;
        let new_key = base64::decode(new_key.private_key_data).context(
            "Unable to base64 decode the private_key_data of the new key returned by google",
        )?;
        let new_key: google::CredentialJson = serde_json::from_slice(&new_key)
            .context("Unable to parse the credentials. Not a valid json object")?;
        *value = serde_json::json!(google::CredentialJsonTypes::Json(new_key));
        Ok(secret_cur)
    }

    async fn set(
        _shared: &'a (),
        _secret_cur: lambda_runtime_types::rotate::SecretContainer<serde_json::Value>,
        _secret_new: lambda_runtime_types::rotate::SecretContainer<serde_json::Value>,
    ) -> anyhow::Result<()> {
        // Not necessary as the `lambda_runtime_types::rotate::RotateRunner::create` already
        // creates the key on google side
        Ok(())
    }

    async fn test(
        _shared: &'a (),
        mut secret_new: lambda_runtime_types::rotate::SecretContainer<serde_json::Value>,
    ) -> anyhow::Result<()> {
        let value = get_mut_json_entry(&mut secret_new)?;
        let credential_json = std::mem::replace(value, serde_json::Value::Null);
        let iam = google::Iam::new(credential_json).await?;
        iam.test_permission().await?;
        Ok(())
    }

    async fn finish(
        _shared: &'a (),
        mut secret_cur: lambda_runtime_types::rotate::SecretContainer<serde_json::Value>,
        mut secret_new: lambda_runtime_types::rotate::SecretContainer<serde_json::Value>,
    ) -> anyhow::Result<()> {
        let value_new = get_mut_json_entry(&mut secret_new)?;
        let credential_json_new = std::mem::replace(value_new, serde_json::Value::Null);
        let iam = google::Iam::new(credential_json_new).await?;

        let value_cur = get_mut_json_entry(&mut secret_cur)?;
        let credential_json_cur = std::mem::replace(value_cur, serde_json::Value::Null);
        let credential_json_cur = google::Iam::parse_json_type(credential_json_cur)?;

        iam.delete_key(&credential_json_cur.private_key_id).await?;
        Ok(())
    }
}

type JsonPath = Vec<JsonPathElement>;

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum JsonPathElement {
    Key(String),
    Index(usize),
}

fn get_mut_json_entry(value: &mut serde_json::Value) -> anyhow::Result<&mut serde_json::Value> {
    use anyhow::Context;

    let json_path = std::env::var(ENV_VAR_JSON_PATH);
    let json_path = json_path.as_deref().unwrap_or("[]");
    let json_path: JsonPath = serde_json::from_str(json_path)
        .with_context(|| format!("Unable to parse JSON_PATH env variable. {} does not conform to the expected json structure.", json_path))?;

    let mut res_value = value;
    for path_element in json_path {
        match path_element {
            JsonPathElement::Key(ref key) => {
                res_value = res_value
                    .get_mut(key)
                    .with_context(|| format!("Json Object does not contain key: {}", key))?;
            }
            JsonPathElement::Index(index) => {
                res_value = res_value
                    .get_mut(index)
                    .with_context(|| format!("Json Array does not contain index: {}", index))?;
            }
        }
    }
    Ok(res_value)
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

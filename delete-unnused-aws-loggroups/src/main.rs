//! This tool removes log groups which are no longer in use
//!
//! # Setup
//!
//! This lambda requires the following IAM Policy to be able to list Cloudwatch LogGroups,
//! Lambdas and CodeBuild Projects, as well as delete CloudWatch LogGroups.
//!
//! ```json
//! {
//!     "Version": "2012-10-17",
//!     "Statement": [
//!         {
//!             "Effect": "Allow",
//!             "Action": [
//!                 "logs:CreateLogStream",
//!                 "logs:PutLogEvents"
//!             ],
//!             "Resource": [
//!                 "arn:aws:logs:{region}:{account_id}:log-group:${lambda_name}:log-stream:*",
//!                 "arn:aws:logs:{region}:{account_id}:log-group:${lambda_name}"
//!             ]
//!         },
//!         {
//!             "Effect": "Allow",
//!             "Action": [
//!                 "logs:DeleteLogGroup",
//!                 "logs:DescribeLogGroups"
//!             ],
//!             "Resource": "*"
//!         },
//!         {
//!             "Effect": "Allow",
//!             "Action": [
//!                 "lambda:ListFunctions"
//!             ],
//!             "Resource": "*"
//!         },
//!         {
//!             "Effect": "Allow",
//!             "Action": [
//!                 "codebuild:ListProjects"
//!             ],
//!             "Resource": "*"
//!         }
//!     ]
//! }
//! ```
//!
//! # Parameters
//!
//! The lambda function has the following parameters. You can define
//! them via environment variables.
//!
//! ## Environment Variables
//! ```sh
//! # Optional, skip if not required. off | error | warn | info (default) | debug | trace
//! # Defines the log level
//! LOG_LEVEL=""
//! ```

#![deny(clippy::all, clippy::nursery)]
#![deny(nonstandard_style, rust_2018_idioms)]

mod aws;

#[cfg(feature = "test")]
const TEST_DATA: &str = include_str!("../test.json");

const ENV_VAR_LOG_LEVEL: &str = "LOG_LEVEL";

struct Runner;

#[async_trait::async_trait]
impl<'a> lambda_runtime_types::Runner<'a, (), (), ()> for Runner {
    async fn run(
        _shared: &'a (),
        event: lambda_runtime_types::LambdaEvent<'a, ()>,
    ) -> anyhow::Result<()> {
        use anyhow::Context;
        use std::str::FromStr;

        let region = rusoto_core::Region::from_str(event.region)
            .with_context(|| format!("Invalid region: {}", event.region))?;
        let cloudwatch = aws::Logs::new(region.clone());
        let lambda = aws::Lambda::new(region.clone());
        let codebuild = aws::CodeBuild::new(region);

        let groups = cloudwatch
            .get_log_groups()
            .await?
            .into_iter()
            .filter_map(|g| g.log_group_name)
            .collect::<Vec<_>>();
        let lambdas = lambda
            .get_functions()
            .await?
            .into_iter()
            .filter_map(|f| f.function_name)
            .collect::<std::collections::HashSet<_>>();
        let projects = codebuild
            .get_projects()
            .await?
            .into_iter()
            .collect::<std::collections::HashSet<_>>();

        for group in groups {
            if let Some(lgroup) = group.strip_prefix("/aws/lambda/") {
                if lambdas.contains(lgroup) {
                    log::debug!("Ignoring LogGroup for Lambda: {} - {}", group, lgroup);
                    continue;
                }
            } else if let Some(cgroup) = group.strip_prefix("/aws/codebuild/") {
                if projects.contains(cgroup) {
                    log::debug!("Ignoring LogGroup for CodeBuild: {} - {}", group, cgroup);
                    continue;
                }
            } else {
                continue;
            }
            log::info!("Deleting LogGroup: {}", group);
            cloudwatch.delete_log_group(&group).await?;
        }
        Ok(())
    }

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

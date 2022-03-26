//! This tool exports CloudWatch LogGroups to S3 once a day
//!
//! # Limitations
//!
//! This lambda is able to backup between 17_280 and 86_400 log groups. The actual value
//! depends on how long it takes to backup a single log group. This restriction is based
//! on the limitation of AWS only allowing a single S3 export at a time per region and
//! account.
//!
//! # Setup
//!
//! ## Lambda IAM Policy
//!
//! This lambda requires the following IAM Policy to be able to list Cloudwatch LogGroups
//! and to list and create export tasks. It also requires permissions to call itself to
//! keep on running until all log groups were exported.
//!
//! ```json
//! {
//!     "Version": "2012-10-17",
//!     "Statement": [
//!         {
//!             "Effect": "Allow",
//!             "Action": [
//!                 "logs:CreateExportTask",
//!                 "logs:DescribeExportTasks",
//!                 "logs:DescribeLogGroups",
//!                 "logs:DescribeLogStreams",
//!                 "logs:ListTagsLogGroup"
//!             ],
//!             "Resource": "*"
//!         },{
//!             "Effect": "Allow",
//!             "Action": [
//!                 "s3:PutObject"
//!             ],
//!             "Resource": "arn:aws:s3:::<bucket_name>/<prefix>/*"
//!         },{
//!            "Effect": "Allow",
//!             "Action": [
//!                 "lambda:InvokeAsync"
//!             ],
//!             "Resource": "<lambda_arn>"
//!         }
//!     ]
//! }
//! ```
//!
//! ## Bucket Policy
//!
//! The bucket also requires a policy to allow exports
//!
//! ```json
//! {
//!     "Version": "2012-10-17",
//!     "Statement": [
//!         {
//!             "Sid": "AWSAclCheck",
//!             "Effect": "Allow",
//!             "Action": "s3:GetBucketAcl",
//!             "Resource": "arn:aws:s3:::<bucket_name>",
//!             "Principal": {
//!                 "Service": [
//!                     "logs.<region>.amazonaws.com"
//!                 ]
//!             },
//!             "Condition": {
//!                 "StringEquals": {
//!                     "aws:SourceAccount": "<SourceAccountID>"
//!                 }
//!             }
//!         },
//!         {
//!             "Sid": "AWSCloudWatchWrite",
//!             "Effect": "Allow",
//!             "Action": "s3:PutObject",
//!             "Resource": "arn:aws:s3:::<bucket_name>/<prefix>/*",
//!             "Principal": {
//!                 "Service": [
//!                     "logs.<region>.amazonaws.com"
//!                 ]
//!             },
//!             "Condition": {
//!                 "StringEquals": {
//!                     "s3:x-amz-acl": "bucket-owner-full-control",
//!                     "aws:SourceAccount": "<SourceAccountID>"
//!                 }
//!             }
//!         },
//!         {
//!             "Sid": "AWSCloudWatchIamWrite",
//!             "Effect": "Allow",
//!             "Action": "s3:PutObject",
//!             "Resource": "arn:aws:s3:::<bucket_name>/<prefix>/*",
//!             "Principal": {
//!                 "AWS": "<lambda_iam_user>"
//!             },
//!             "Condition": {
//!                 "StringEquals": {
//!                     "s3:x-amz-acl": "bucket-owner-full-control"
//!                 }
//!             }
//!         }
//!     ]
//! }
//! ```
//!
//! ## Lambda Configuration
//!
//! For the correct operation of the lambda function two configurations are required:
//! * Lambda timeout: 15 minutes
//! * Lambda execution cron: 12pm UTC
//!
//! The function automatically calls itself after 10 minutes if there are more LogGroups
//! to backup (it will run at most 24h to not interfere with the next backup round). For
//! this to work the timeout must be greater then 10 minutes to allow the recursion to
//! happen.
//!
//! The execution time is imporant as this lambda will always backup logs from 12:00am UTC to
//! 11:59pm UTC. The [AWS docu](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/S3Export.html)
//! states that it may take up to 12 hours for logs to be ready for export. That is why the
//! lambda should run 12 hours after the creation of the last log entry that is to be exported.
//!
//! # Parameters
//!
//! The lambda function requires a few parameters to correctly work. You can define
//! them either with the Event that is send to the lambda, or via environment variables.
//!
//! The `bucket_prefix` allows these variables:
//! * `{region}`: Region of the log group
//! * `{group}`: Name of the log group
//! * `{year}`: Year of the log entry creation
//! * `{month}`: Month of the log entry creation
//! * `{day}`: Day of the log entry creation
//!
//! ## Event
//!
//! ```js
//! {
//!     "bucket": "<bucket_name>",
//!     "prefix": "<bucket_prefix>",
//!     // Optional, skip if not required. Example: `tag1=value1,tag2=value2`
//!     // Only include cloudwatch groups with the given tags and values
//!     "include_tags": [{
//!         "name": "",
//!         "value": ""
//!     }],
//!     // Optional, skip if not required. Example: `tag1=value1,tag2=value2`
//!     // Exclude cloudwatch groups with the given tags and values
//!     "exclude_tags": [{
//!         "name": "",
//!         "value": ""
//!     }]
//! }
//! ```
//!
//! ## Environment Variables
//! ```sh
//! BUCKET="<bucket_name>"
//! PREFIX="<bucket_prefix>"
//! # Optional, skip if not required. off | error | warn | info (default) | debug | trace
//! # Defines the log level
//! LOG_LEVEL=""
//! # Optional, skip if not required. Example: `tag1=value1,tag2=value2`
//! # Only include cloudwatch groups with the given tags and values
//! INCLUDE_TAGS=""
//! # Optional, skip if not required. Example: `tag1=value1,tag2=value2`
//! # Exclude cloudwatch groups with the given tags and values
//! EXCLUDE_TAGS=""
//! ```

#![deny(clippy::all, clippy::nursery)]
#![deny(nonstandard_style, rust_2018_idioms)]

mod aws;
mod event;

#[cfg(feature = "test")]
const TEST_DATA: &str = include_str!("../test.json");

const ENV_VAR_LOG_LEVEL: &str = "LOG_LEVEL";
const ENV_VAR_BUCKET: &str = "BUCKET";
const ENV_VAR_PREFIX: &str = "PREFIX";
const ENV_VAR_INCLUDE_TAGS: &str = "INCLUDE_TAGS";
const ENV_VAR_EXCLUDE_TAGS: &str = "EXCLUDE_TAGS";

const EXPORT_TASK_PREFIX: &str = "AUTO-EXPORT-";

struct Runner;

enum ExportTaskStatus {
    Cancelled,
    Completed,
    Failed,
    Pending,
    PendingCancel,
    Running,
}

impl ExportTaskStatus {
    pub fn try_convert(s: &str) -> Option<Self> {
        match s {
            "CANCELLED" => Some(Self::Cancelled),
            "COMPLETED" => Some(Self::Completed),
            "FAILED" => Some(Self::Failed),
            "PENDING" => Some(Self::Pending),
            "PENDING_CANCEL" => Some(Self::PendingCancel),
            "RUNNING" => Some(Self::Running),
            _ => None,
        }
    }
}

struct ExportTask {
    log_group_name: String,
}

impl ExportTask {
    pub fn try_convert(
        task: rusoto_logs::ExportTask,
        not_before: chrono::NaiveDateTime,
        not_after: chrono::NaiveDateTime,
    ) -> Option<Self> {
        if task.from? < not_before.timestamp_millis() || task.to? > not_after.timestamp_millis() {
            return None;
        }
        if !task.task_name?.starts_with(EXPORT_TASK_PREFIX) {
            return None;
        }
        match ExportTaskStatus::try_convert(&task.status?.code?)? {
            ExportTaskStatus::Cancelled
            | ExportTaskStatus::Failed
            | ExportTaskStatus::PendingCancel => return None,
            _ => {}
        }
        let log_group_name = task.log_group_name?;
        Some(Self { log_group_name })
    }
}

#[async_trait::async_trait]
impl<'a> lambda_runtime_types::Runner<'a, (), event::Event, ()> for Runner {
    async fn run(
        _shared: &'a (),
        event: lambda_runtime_types::LambdaEvent<'a, event::Event>,
    ) -> anyhow::Result<()> {
        use anyhow::Context;
        use chrono::Datelike;
        use std::str::FromStr;

        // Start and end time of lambda
        let invoke_time_local = chrono::Utc::now().naive_utc();
        let end_time_local = invoke_time_local + chrono::Duration::minutes(10);

        // Start and end time of backup process
        let invoke_time_backup = event.event.invoke_time.unwrap_or(invoke_time_local);
        let end_time_backup =
            invoke_time_backup + chrono::Duration::days(1) - chrono::Duration::minutes(5);

        // Start and end time of export
        let export_date = invoke_time_backup.date() - chrono::Duration::days(1);
        let export_start_time = chrono::NaiveTime::from_hms_milli(0, 0, 0, 0);
        let export_end_time = chrono::NaiveTime::from_hms_milli(23, 59, 59, 999);
        let export_start = chrono::NaiveDateTime::new(export_date, export_start_time);
        let export_end = chrono::NaiveDateTime::new(export_date, export_end_time);

        let prefix = event
            .event
            .get_prefix()?
            .replace("{year}", &format!("{:02}", export_date.year()))
            .replace("{month}", &format!("{:02}", export_date.month()))
            .replace("{day}", &format!("{:02}", export_date.day()));

        let bucket = event.event.get_bucket()?;
        let prefix = prefix.replace("{region}", event.region);
        let include_tags = event.event.get_include_tags()?;
        let exclude_tags = event.event.get_exclude_tags()?;

        let region = rusoto_core::Region::from_str(event.region)
            .with_context(|| format!("Invalid region: {}", event.region))?;
        let cloudwatch = aws::Logs::new(region.clone());
        let lambda = aws::Lambda::new(region);

        let mut groups = cloudwatch
            .get_log_groups()
            .await?
            .into_iter()
            .filter_map(|g| g.log_group_name)
            .collect::<std::collections::HashSet<_>>();
        let export_tasks = cloudwatch
            .get_export_tasks()
            .await?
            .into_iter()
            .filter_map(|t| ExportTask::try_convert(t, export_start, export_end))
            .collect::<Vec<_>>();

        for export_task in export_tasks {
            groups.remove(&export_task.log_group_name);
        }
        for group in groups {
            let now = chrono::Utc::now().naive_utc();
            if now >= end_time_backup {
                break;
            }
            if now >= end_time_local {
                lambda
                    .invoke_async(
                        &event.ctx.invoked_function_arn,
                        &event::Event {
                            invoke_time: Some(invoke_time_backup),
                            ..event.event
                        },
                    )
                    .await?;
                break;
            }
            println!("Querying info for LogGroup {}", group);
            if include_tags.is_some() || exclude_tags.is_some() {
                let tags = cloudwatch.get_tags(&group).await?;
                if let Some(ref include_tags) = include_tags {
                    if !include_tags
                        .iter()
                        .any(|tag| tags.get(&tag.name) == Some(&tag.value))
                    {
                        println!("Skipping LogGroup {} as it is missing required tags", group);
                        continue;
                    }
                }
                if let Some(ref exclude_tags) = exclude_tags {
                    if exclude_tags
                        .iter()
                        .any(|tag| tags.get(&tag.name) == Some(&tag.value))
                    {
                        println!(
                            "Skipping LogGroup {} as has a tag on the excluded lists",
                            group
                        );
                        continue;
                    }
                }
            }
            if cloudwatch
                .get_last_event_timestamp(&group)
                .await?
                .map(|t| t < export_start.timestamp_millis())
                .unwrap_or(true)
            {
                println!("Skipping LogGroup {} due to no new log entries", group);
                continue;
            }

            println!("Trying to create export for LogGroup {}", group);
            let prefix = prefix.replace("{group}", group.trim_start_matches('/'));
            let task_name = generate_task_name();
            let task_id = cloudwatch
                .create_export_tasks(
                    &bucket,
                    &prefix,
                    export_start,
                    &group,
                    export_end,
                    &task_name,
                )
                .await?;
            println!(
                "Created export for LogGroup {} with name {} and id {}",
                group, task_name, task_id
            );
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

fn generate_task_name() -> String {
    use rand::Rng;

    let mut task_name = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(7)
        .map(char::from)
        .collect::<String>();
    task_name.insert_str(0, EXPORT_TASK_PREFIX);
    task_name
}

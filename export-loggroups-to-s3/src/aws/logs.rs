#[derive(Clone)]
pub struct Logs {
    client: rusoto_logs::CloudWatchLogsClient,
}

impl std::fmt::Debug for Logs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Logs").field("client", &"[...]").finish()
    }
}

impl Logs {
    pub(crate) fn new(region: rusoto_core::Region) -> Self {
        Self {
            client: rusoto_logs::CloudWatchLogsClient::new(region),
        }
    }

    pub(crate) async fn get_log_groups(&self) -> anyhow::Result<Vec<rusoto_logs::LogGroup>> {
        use anyhow::Context;
        use rusoto_logs::CloudWatchLogs;

        let mut next_token = None;
        let mut groups = Vec::new();
        loop {
            let res = self
                .client
                .describe_log_groups(rusoto_logs::DescribeLogGroupsRequest {
                    next_token: next_token.clone(),
                    ..rusoto_logs::DescribeLogGroupsRequest::default()
                })
                .await;
            if super::is_wait_and_repeat(&res).await {
                continue;
            }
            let res = res.context("Unable to fetch log groups")?;
            if let Some(log_groups) = res.log_groups {
                groups.extend(log_groups);
            }
            if let Some(token) = res.next_token {
                next_token = Some(token);
                continue;
            }
            break;
        }
        Ok(groups)
    }

    pub(crate) async fn get_tags(
        &self,
        group_name: &str,
    ) -> anyhow::Result<std::collections::HashMap<String, String>> {
        use anyhow::Context;
        use rusoto_logs::CloudWatchLogs;

        loop {
            let res = self
                .client
                .list_tags_log_group(rusoto_logs::ListTagsLogGroupRequest {
                    log_group_name: group_name.into(),
                })
                .await;
            if super::is_wait_and_repeat(&res).await {
                continue;
            }
            let res = res.context("Unable to fetch log groups")?;
            break Ok(res.tags.unwrap_or_default());
        }
    }

    pub(crate) async fn get_last_event_timestamp(
        &self,
        group_name: &str,
    ) -> anyhow::Result<Option<i64>> {
        use anyhow::Context;
        use rusoto_logs::CloudWatchLogs;

        let mut next_token = None;
        loop {
            let res = self
                .client
                .describe_log_streams(rusoto_logs::DescribeLogStreamsRequest {
                    descending: Some(true),
                    log_group_name: group_name.into(),
                    next_token: next_token.clone(),
                    order_by: Some("LastEventTime".into()),
                    ..rusoto_logs::DescribeLogStreamsRequest::default()
                })
                .await;
            if super::is_wait_and_repeat(&res).await {
                continue;
            }
            let res = res.context("Unable to fetch log streams")?;
            for stream in res.log_streams.unwrap_or_default() {
                if let Some(timestamp) = stream.last_event_timestamp {
                    return Ok(Some(timestamp));
                }
            }
            if let Some(token) = res.next_token {
                next_token = Some(token);
                continue;
            }
            return Ok(None);
        }
    }

    pub(crate) async fn get_export_tasks(&self) -> anyhow::Result<Vec<rusoto_logs::ExportTask>> {
        use anyhow::Context;
        use rusoto_logs::CloudWatchLogs;

        let mut next_token = None;
        let mut tasks = Vec::new();
        loop {
            let res = self
                .client
                .describe_export_tasks(rusoto_logs::DescribeExportTasksRequest {
                    next_token: next_token.clone(),
                    ..rusoto_logs::DescribeExportTasksRequest::default()
                })
                .await;
            if super::is_wait_and_repeat(&res).await {
                continue;
            }
            let res = res.context("Unable to fetch log groups")?;
            if let Some(export_tasks) = res.export_tasks {
                tasks.extend(export_tasks);
            }
            if let Some(token) = res.next_token {
                next_token = Some(token);
                continue;
            }
            break;
        }
        Ok(tasks)
    }

    pub(crate) async fn create_export_tasks(
        &self,
        bucket: &str,
        prefix: &str,
        from: chrono::NaiveDateTime,
        group_name: &str,
        to: chrono::NaiveDateTime,
        task_name: &str,
    ) -> anyhow::Result<String> {
        use anyhow::Context;
        use rusoto_logs::CloudWatchLogs;

        loop {
            let res = self
                .client
                .create_export_task(rusoto_logs::CreateExportTaskRequest {
                    destination: bucket.into(),
                    destination_prefix: Some(prefix.into()),
                    from: from.timestamp_millis(),
                    log_group_name: group_name.into(),
                    task_name: Some(task_name.into()),
                    to: to.timestamp_millis(),
                    ..rusoto_logs::CreateExportTaskRequest::default()
                })
                .await;
            if super::is_wait_and_repeat(&res).await {
                continue;
            }
            if let Err(rusoto_core::RusotoError::Service(
                rusoto_logs::CreateExportTaskError::LimitExceeded(_),
            )) = res
            {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                continue;
            }
            let res = res.context("Unable to create log group export")?;
            if let Some(task_id) = res.task_id {
                return Ok(task_id);
            }
        }
    }
}

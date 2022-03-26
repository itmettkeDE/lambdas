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

    pub(crate) async fn delete_log_group(&self, name: &str) -> anyhow::Result<()> {
        use anyhow::Context;
        use rusoto_logs::CloudWatchLogs;

        loop {
            let res = self
                .client
                .delete_log_group(rusoto_logs::DeleteLogGroupRequest {
                    log_group_name: name.into(),
                })
                .await;
            if super::is_wait_and_repeat(&res).await {
                continue;
            }
            res.context("Unable to fetch log groups")?;
            break;
        }
        Ok(())
    }
}

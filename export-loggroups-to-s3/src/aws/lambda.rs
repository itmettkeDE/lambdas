#[derive(Clone)]
pub struct Lambda {
    client: rusoto_lambda::LambdaClient,
}

impl std::fmt::Debug for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Logs").field("client", &"[...]").finish()
    }
}

impl Lambda {
    pub(crate) fn new(region: rusoto_core::Region) -> Self {
        Self {
            client: rusoto_lambda::LambdaClient::new(region),
        }
    }

    pub(crate) async fn invoke_async(
        &self,
        function_name: &str,
        args: &crate::event::Event,
    ) -> anyhow::Result<()> {
        use anyhow::Context;
        use rusoto_lambda::Lambda;

        let data = serde_json::to_vec(args).context("Unable to serialize event")?;
        loop {
            let res = self
                .client
                .invoke_async(rusoto_lambda::InvokeAsyncRequest {
                    function_name: function_name.into(),
                    invoke_args: data.clone().into(),
                })
                .await;
            if super::is_wait_and_repeat(&res).await {
                continue;
            }
            let res = res.context("Unable to create log group export")?;
            if res.status == Some(202) {
                return Ok(());
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await
        }
    }
}

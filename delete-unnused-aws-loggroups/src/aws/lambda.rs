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

    pub(crate) async fn get_functions(
        &self,
    ) -> anyhow::Result<Vec<rusoto_lambda::FunctionConfiguration>> {
        use anyhow::Context;
        use rusoto_lambda::Lambda;

        let mut marker = None;
        let mut functions = Vec::new();
        loop {
            let res = self
                .client
                .list_functions(rusoto_lambda::ListFunctionsRequest {
                    marker: marker.clone(),
                    ..rusoto_lambda::ListFunctionsRequest::default()
                })
                .await;
            if super::is_wait_and_repeat(&res).await {
                continue;
            }
            let res = res.context("Unable to fetch lambdas")?;
            if let Some(new_functions) = res.functions {
                functions.extend(new_functions);
            }
            if let Some(next_marker) = res.next_marker {
                marker = Some(next_marker);
                continue;
            }
            break;
        }
        Ok(functions)
    }
}

/// Secret Manager Client
#[derive(Clone)]
pub(crate) struct Smc {
    client: rusoto_secretsmanager::SecretsManagerClient,
}

impl std::fmt::Debug for Smc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Smc").field("client", &"[...]").finish()
    }
}

impl Smc {
    /// Create a new secret manager client
    pub(crate) fn new(region: rusoto_core::Region) -> Self {
        Self {
            client: rusoto_secretsmanager::SecretsManagerClient::new(region),
        }
    }

    /// Fetches the current secret value of the given secret_id
    pub(crate) async fn get_secret_value_current<S: serde::de::DeserializeOwned>(
        &self,
        secret_id: &str,
    ) -> anyhow::Result<S> {
        use anyhow::Context;
        use rusoto_secretsmanager::SecretsManager;

        let secret_value = loop {
            let res = self
                .client
                .get_secret_value(rusoto_secretsmanager::GetSecretValueRequest {
                    secret_id: secret_id.to_string(),
                    version_id: None,
                    version_stage: Some("AWSCURRENT".to_string()),
                })
                .await;
            if Self::is_wait_and_repeat(&res).await {
                continue;
            }
            break res
                .with_context(|| format!("Unable to fetch SecretValue with id: {}", secret_id))?;
        };
        let inner = match (secret_value.secret_string, secret_value.secret_binary) {
            (Some(string), _) => serde_json::from_str(&string),
            (_, Some(bytes)) => serde_json::from_slice(&bytes),
            _ => anyhow::bail!("Neither secret_string nor secret_binary is set for id: {}", secret_id),
        }
        .with_context(|| format!("Unable to parse secret value. Value does not confirm to required structure. Id: {}", secret_id))?;
        Ok(inner)
    }

    /// Checks whether the given result is a throttling error
    /// and waits for 250 ms if it is
    async fn is_wait_and_repeat<D: Send + Sync, E: std::fmt::Debug + Send + Sync>(
        error: &Result<D, rusoto_core::RusotoError<E>>,
    ) -> bool {
        if let Err(rusoto_core::RusotoError::Unknown(
            rusoto_core::request::BufferedHttpResponse {
                ref status,
                ref body,
                ..
            },
        )) = *error
        {
            let cooldown = match status.as_u16() {
                400 => {
                    let search = b"ThrottlingException";
                    body.as_ref().windows(search.len()).any(|sub| sub == search)
                }
                429 => {
                    let search = b"Too Many Requests";
                    body.as_ref().windows(search.len()).any(|sub| sub == search)
                }
                _ => false,
            };
            if cooldown {
                println!("Info: Cooling down to prevent request limits");
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                return true;
            }
        }
        false
    }
}

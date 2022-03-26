const ENDPOINT: &str = "https://iam.googleapis.com/v1";
const SCOPES: &str = "https://www.googleapis.com/auth/iam";

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub(crate) struct Permissions {
    permissions: Vec<std::borrow::Cow<'static, str>>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Credentials {
    #[serde(rename = "privateKeyData")]
    pub(crate) private_key_data: String,
}

#[derive(Debug, serde::Deserialize)]
struct AuthToken {
    access_token: String,
}

#[derive(Debug, serde::Serialize)]
struct JwtClaims<'a> {
    iss: &'a str,
    sub: &'a str,
    scope: &'a str,
    aud: &'a str,
    exp: u64,
    iat: u64,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
#[allow(variant_size_differences)]
pub(crate) enum CredentialJsonTypes {
    String(String),
    Json(CredentialJson),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct CredentialJson {
    pub(crate) private_key_id: PrivateKeyId,
    private_key: String,
    token_uri: String,
    client_email: String,
    #[serde(flatten)]
    _o: std::collections::HashMap<String, serde_json::Value>,
}

#[repr(transparent)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct PrivateKeyId(pub(crate) String);

#[derive(Debug)]
pub(crate) struct Iam {
    client: reqwest::Client,
    token: String,
    client_email: String,
}

impl Iam {
    pub(crate) async fn new(secret: serde_json::Value) -> anyhow::Result<Self> {
        let client = reqwest::Client::new();
        let credential_json = Self::parse_json_type(secret)?;
        let jwt = Self::sign_jwt(&credential_json)?;
        let token = Self::fetch_token_by_jwt(jwt, &credential_json.token_uri, &client).await?;
        Ok(Self {
            client,
            token,
            client_email: credential_json.client_email.clone(),
        })
    }

    pub(crate) fn parse_json_type(json_type: serde_json::Value) -> anyhow::Result<CredentialJson> {
        use anyhow::Context;
        let json_type: CredentialJsonTypes = serde_json::from_value(json_type)?;

        Ok(match json_type {
            CredentialJsonTypes::Json(json) => json,
            CredentialJsonTypes::String(string) => serde_json::from_str::<CredentialJson>(&string)
                .context("Unable to parse credential_json")?,
        })
    }

    fn sign_jwt(credential_json: &CredentialJson) -> anyhow::Result<String> {
        use anyhow::Context;

        let timestamp = get_current_timestamp();
        let exp_time = timestamp + (15 * 60);
        let header = jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::RS256,
            kid: Some(credential_json.private_key_id.0.clone()),
            ..jsonwebtoken::Header::default()
        };
        let claims = JwtClaims {
            iss: &credential_json.client_email,
            sub: &credential_json.client_email,
            scope: SCOPES,
            aud: &credential_json.token_uri,
            exp: exp_time,
            iat: timestamp,
        };
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(credential_json.private_key.as_bytes())
            .context("Unable to decode private_key in Google Credentials.")?;
        jsonwebtoken::encode(&header, &claims, &key)
            .context("Unable to sign jwt token with Google Credentials")
    }

    async fn fetch_token_by_jwt(
        jwt: String,
        token_url: &str,
        client: &reqwest::Client,
    ) -> anyhow::Result<String> {
        use anyhow::Context;

        Ok(client
            .request(reqwest::Method::POST, token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .context("Unable to send request to Google Auth Url")?
            .error_for_status()
            .context("Error returned from server")?
            .json::<AuthToken>()
            .await
            .map(|d| d.access_token)
            .context("Could not parse result from Google Auth Url")?)
    }

    pub(crate) async fn test_permission(&self) -> anyhow::Result<()> {
        use anyhow::{bail, Context};

        let permissions_expected = Permissions {
            permissions: vec![
                "iam.serviceAccountKeys.create".into(),
                "iam.serviceAccountKeys.delete".into(),
            ],
        };
        let permissions_granted = self
            .client
            .request(
                reqwest::Method::POST,
                &format!(
                    "{}/projects/-/serviceAccounts/{}:testIamPermissions",
                    ENDPOINT, self.client_email
                ),
            )
            .header("Authorization", format!("Bearer {}", &self.token))
            .header("Content-Type", "application/json")
            .json(&permissions_expected)
            .send()
            .await
            .context("Unable to send request to Google IAM API")?
            .error_for_status()
            .context("Error returned from server")?
            .json::<Permissions>()
            .await
            .context("Could not parse result from Google IAM API")?;
        if permissions_expected != permissions_granted {
            bail!("New credentials do not have the required permissions for further rotations.\nRequired: {:?}\nGranted: {:?}", permissions_expected, permissions_granted);
        }
        Ok(())
    }

    pub(crate) async fn create_key(&self) -> anyhow::Result<Credentials> {
        use anyhow::Context;

        self.client
            .request(
                reqwest::Method::POST,
                &format!(
                    "{}/projects/-/serviceAccounts/{}/keys",
                    ENDPOINT, self.client_email
                ),
            )
            .header("Authorization", format!("Bearer {}", &self.token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "keyAlgorithm": "KEY_ALG_RSA_2048",
                "privateKeyType": "TYPE_GOOGLE_CREDENTIALS_FILE"
            }))
            .send()
            .await
            .context("Unable to send request to Google IAM API")?
            .error_for_status()
            .context("Error returned from server")?
            .json::<Credentials>()
            .await
            .context("Could not parse result from Google IAM API")
    }

    pub(crate) async fn delete_key(&self, private_key_id: &PrivateKeyId) -> anyhow::Result<()> {
        use anyhow::Context;

        let _ = self
            .client
            .request(
                reqwest::Method::DELETE,
                &format!(
                    "{}/projects/-/serviceAccounts/{}/keys/{}",
                    ENDPOINT, self.client_email, private_key_id.0
                ),
            )
            .header("Authorization", format!("Bearer {}", &self.token))
            .send()
            .await
            .context("Unable to send request to Google IAM API")?
            .error_for_status()
            .context("Error returned from server")?;
        Ok(())
    }
}

fn get_current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .expect("Current time is prior to unix epoch")
}

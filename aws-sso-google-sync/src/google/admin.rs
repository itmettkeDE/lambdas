const ENDPOINT: &str = "https://admin.googleapis.com/admin/directory/v1";
const SCOPES: &str = "https://www.googleapis.com/auth/admin.directory.group.readonly https://www.googleapis.com/auth/admin.directory.group.member.readonly https://www.googleapis.com/auth/admin.directory.user.readonly";

#[derive(Debug, serde::Deserialize)]
struct Members {
    members: Option<Vec<Member>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Member {
    pub(crate) email: String,
    #[serde(rename = "type")]
    pub(crate) r#type: MemberType,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
pub(crate) enum MemberType {
    #[serde(rename = "USER")]
    User,
    #[serde(other)]
    Other,
}

#[derive(Debug, serde::Deserialize)]
struct Groups {
    groups: Option<Vec<Group>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Group {
    pub(crate) id: String,
    pub(crate) email: String,
}

#[derive(Debug, serde::Deserialize)]
struct Users {
    users: Option<Vec<User>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct User {
    pub(crate) id: String,
    #[serde(rename = "primaryEmail")]
    pub(crate) primary_email: String,
    pub(crate) suspended: Option<bool>,
    pub(crate) name: UserName,
    pub(crate) emails: Vec<UserMail>,
    #[serde(rename = "thumbnailPhotoUrl")]
    pub(crate) thumbnail_photo_url: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct UserName {
    #[serde(rename = "fullName")]
    pub(crate) full_name: String,
    #[serde(rename = "familyName")]
    pub(crate) family_name: String,
    #[serde(rename = "givenName")]
    pub(crate) given_name: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct UserMail {
    pub(crate) address: String,
    pub(crate) primary: Option<bool>,
    #[serde(rename = "type")]
    pub(crate) r#type: Option<String>,
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

#[derive(Debug, serde::Deserialize)]
pub(crate) struct AdminCreds {
    mail: String,
    credential_json: CredentialJsonTypes,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
#[allow(variant_size_differences)]
pub(crate) enum CredentialJsonTypes {
    String(String),
    Json(CredentialJson),
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct CredentialJson {
    private_key_id: String,
    private_key: String,
    token_uri: String,
    client_email: String,
}

#[derive(Debug)]
pub(crate) struct Admin<'a> {
    client: reqwest::Client,
    token: String,
    domain: &'a str,
}

impl<'a> Admin<'a> {
    pub(crate) async fn new(secret: &'a AdminCreds) -> anyhow::Result<Admin<'a>> {
        use anyhow::Context;

        let client = reqwest::Client::new();
        let credential_json;
        let credential_json_ref = match secret.credential_json {
            CredentialJsonTypes::Json(ref v) => v,
            CredentialJsonTypes::String(ref json) => {
                credential_json =
                    serde_json::from_str(json).context("Unable to parse credential_json")?;
                &credential_json
            }
        };
        let jwt = Self::sign_jwt(&secret.mail, credential_json_ref)?;
        let token = Self::fetch_token_by_jwt(jwt, &credential_json_ref.token_uri, &client).await?;
        let domain = secret
            .mail
            .split('@')
            .nth(1)
            .with_context(|| format!("Mail is invalid: {}", secret.mail))?;
        Ok(Self {
            client,
            token,
            domain,
        })
    }

    fn sign_jwt(mail: &str, credential_json: &CredentialJson) -> anyhow::Result<String> {
        use anyhow::Context;

        let timestamp = get_current_timestamp();
        let exp_time = timestamp + (15 * 60);
        let header = jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::RS256,
            kid: Some(credential_json.private_key_id.clone()),
            ..jsonwebtoken::Header::default()
        };
        let claims = JwtClaims {
            iss: &credential_json.client_email,
            sub: mail,
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

    pub(crate) async fn list_users(
        &self,
        query: Option<&str>,
        deleted: bool,
    ) -> anyhow::Result<Vec<User>> {
        use anyhow::Context;

        let mut entries = Vec::new();
        let mut token = None;
        let mut queries = Vec::new();
        loop {
            queries.clear();
            if let Some(token) = token {
                queries.push(("pageToken", token));
            }
            if let Some(query) = query {
                queries.push(("query", String::from(query)));
            }
            let res = self
                .client
                .request(
                    reqwest::Method::GET,
                    &format!(
                        "{}/users?domain={}&showDeleted={}",
                        ENDPOINT, self.domain, deleted
                    ),
                )
                .query(&queries)
                .header("Authorization", format!("Bearer {}", &self.token))
                .header("Accept", "application/json")
                .send()
                .await
                .context("Unable to send request to Google Admin API")?
                .error_for_status()
                .context("Error returned from server")?
                .json::<Users>()
                .await
                .context("Could not parse result from Google Admin API")?;

            if let Some(mut new_entries) = res.users {
                entries.append(&mut new_entries);
            }
            if let Some(next_page_token) = res.next_page_token {
                token = Some(next_page_token);
                continue;
            }
            break;
        }

        Ok(entries)
    }

    pub(crate) async fn list_groups(&self, query: Option<&str>) -> anyhow::Result<Vec<Group>> {
        use anyhow::Context;

        let mut entries = Vec::new();
        let mut token = None;
        let mut queries = Vec::new();
        loop {
            queries.clear();
            if let Some(token) = token {
                queries.push(("pageToken", token));
            }
            if let Some(query) = query {
                queries.push(("query", String::from(query)));
            }
            let res = self
                .client
                .request(
                    reqwest::Method::GET,
                    &format!("{}/groups?domain={}", ENDPOINT, self.domain),
                )
                .query(&queries)
                .header("Authorization", format!("Bearer {}", &self.token))
                .header("Accept", "application/json")
                .send()
                .await
                .context("Unable to send request to Google Admin API")?
                .error_for_status()
                .context("Error returned from server")?
                .json::<Groups>()
                .await
                .context("Could not parse result from Google Admin API")?;

            if let Some(mut new_entries) = res.groups {
                entries.append(&mut new_entries);
            }
            if let Some(next_page_token) = res.next_page_token {
                token = Some(next_page_token);
                continue;
            }
            break;
        }

        Ok(entries)
    }

    pub(crate) async fn list_group_members(
        &self,
        group_id: &str,
    ) -> anyhow::Result<std::collections::HashSet<String>> {
        use anyhow::Context;

        let mut entries = std::collections::HashSet::new();
        let mut token: Option<String> = None;
        let mut query: [_; 1] = [("", String::new())];
        loop {
            let query: &[_] = token.map_or_else(
                || &[][..],
                |token| {
                    query = [("pageToken", token)];
                    &query
                },
            );
            let res = self
                .client
                .request(
                    reqwest::Method::GET,
                    &format!(
                        "{}/groups/{}/members?includeDerivedMembership=true",
                        ENDPOINT, group_id
                    ),
                )
                .query(query)
                .header("Authorization", format!("Bearer {}", &self.token))
                .header("Accept", "application/json")
                .send()
                .await
                .context("Unable to send request to Google Admin API")?
                .error_for_status()
                .context("Error returned from server")?
                .json::<Members>()
                .await
                .context("Could not parse result from Google Admin API")?;

            if let Some(new_entries) = res.members {
                entries.extend(
                    new_entries
                        .into_iter()
                        .filter(|e| e.r#type == MemberType::User)
                        .map(|e| e.email),
                );
            }
            if let Some(next_page_token) = res.next_page_token {
                token = Some(next_page_token);
                continue;
            }
            break;
        }

        Ok(entries)
    }
}

fn get_current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .expect("Current time is prior to unix epoch")
}

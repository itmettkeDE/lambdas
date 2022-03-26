#[derive(Debug, serde::Deserialize)]
struct ListResponse<Inner> {
    #[serde(rename = "Resources")]
    resources: Vec<Inner>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Group {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    #[serde(rename = "displayName")]
    pub(crate) display_name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    #[serde(rename = "externalId")]
    pub(crate) external_id: Option<String>,
    #[serde(rename = "userName")]
    pub(crate) user_name: String,
    pub(crate) name: UserName,
    #[serde(rename = "displayName")]
    pub(crate) display_name: String,
    #[serde(rename = "profileUrl", skip_serializing_if = "Option::is_none")]
    pub(crate) profile_url: Option<String>,
    pub(crate) emails: Option<Vec<UserMail>>,
    pub(crate) active: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct UserName {
    #[serde(rename = "formatted", skip_serializing_if = "Option::is_none")]
    pub(crate) formatted: Option<String>,
    #[serde(rename = "familyName")]
    pub(crate) family_name: String,
    #[serde(rename = "givenName")]
    pub(crate) given_name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct UserMail {
    pub(crate) value: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub(crate) r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) primary: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ScimCreds {
    endpoint: String,
    access_token: String,
}

#[derive(Debug)]
pub(crate) struct Scim<'a> {
    secret: &'a ScimCreds,
    client: reqwest::Client,
}

impl<'a> Scim<'a> {
    pub(crate) fn new(secret: &'a ScimCreds) -> Self {
        Self {
            secret,
            client: reqwest::Client::new(),
        }
    }

    pub(crate) async fn list_users(&self) -> anyhow::Result<Vec<User>> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::GET,
                    &format!("{}/Users", &self.secret.endpoint),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .send()
                .await
                .context("Unable to send request to AWS SCIM (list_users)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            return res
                .error_for_status()
                .context("Error returned from server (list_users)")?
                .json::<ListResponse<User>>()
                .await
                .map(|d| d.resources)
                .context("Could not parse result from AWS SCIM (list_users)");
        }
    }

    pub(crate) async fn get_user(&self, primary_email: &str) -> anyhow::Result<Option<User>> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::GET,
                    &format!("{}/Users?userName={}", &self.secret.endpoint, primary_email),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .send()
                .await
                .context("Unable to send request to AWS SCIM (get_user)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            if res.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            return res
                .error_for_status()
                .context("Error returned from server (get_user)")?
                .json::<ListResponse<User>>()
                .await
                .context("Could not parse result from AWS SCIM (get_user)")
                .map(|mut d| d.resources.pop());
        }
    }

    pub(crate) async fn create_user(&self, user: User) -> anyhow::Result<Option<User>> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::POST,
                    &format!("{}/Users", &self.secret.endpoint),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .json(&user)
                .send()
                .await
                .context("Unable to send request to AWS SCIM (create_user)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            if res.status() == reqwest::StatusCode::CONFLICT {
                return Ok(None);
            }
            return res
                .error_for_status()
                .context("Error returned from server (create_user)")?
                .json::<User>()
                .await
                .context("Could not parse result from AWS SCIM (create_user)")
                .map(Some);
        }
    }

    pub(crate) async fn delete_user(&self, user_id: &str) -> anyhow::Result<()> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::DELETE,
                    &format!("{}/Users/{}", &self.secret.endpoint, user_id),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .send()
                .await
                .context("Unable to send request to AWS SCIM (delete_user)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            let _ = res
                .error_for_status()
                .context("Error returned from server (delete_user)")?;
            return Ok(());
        }
    }

    pub(crate) async fn list_groups(&self) -> anyhow::Result<Vec<Group>> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::GET,
                    &format!("{}/Groups", &self.secret.endpoint),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .send()
                .await
                .context("Unable to send request to AWS SCIM (list_groups)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            return res
                .error_for_status()
                .context("Error returned from server (list_groups)")?
                .json::<ListResponse<Group>>()
                .await
                .map(|d| d.resources)
                .context("Could not parse result from AWS SCIM (list_groups)");
        }
    }

    pub(crate) async fn get_group(&self, display_name: &str) -> anyhow::Result<Group> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::GET,
                    &format!(
                        "{}/Groups?displayName={}",
                        &self.secret.endpoint, display_name
                    ),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .send()
                .await
                .context("Unable to send request to AWS SCIM (get_group)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            return res
                .error_for_status()
                .context("Error returned from server (get_group)")?
                .json::<ListResponse<Group>>()
                .await
                .context("Could not parse result from AWS SCIM (get_group)")
                .and_then(|mut d| {
                    d.resources.pop().with_context(|| {
                        format!("Unable to find group with name: {}", display_name)
                    })
                });
        }
    }

    pub(crate) async fn create_group(&self, group: Group) -> anyhow::Result<Option<Group>> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::POST,
                    &format!("{}/Groups", &self.secret.endpoint),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .json(&group)
                .send()
                .await
                .context("Unable to send request to AWS SCIM (create_group)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            if res.status() == reqwest::StatusCode::CONFLICT {
                return Ok(None);
            }
            return res
                .error_for_status()
                .context("Error returned from server (create_group)")?
                .json::<Group>()
                .await
                .context("Could not parse result from AWS SCIM (create_group)")
                .map(Some);
        }
    }

    pub(crate) async fn delete_group(&self, group_id: &str) -> anyhow::Result<()> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::DELETE,
                    &format!("{}/Groups/{}", &self.secret.endpoint, group_id),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .send()
                .await
                .context("Unable to send request to AWS SCIM (delete_group)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            let _ = res
                .error_for_status()
                .context("Error returned from server (delete_group)")?;
            return Ok(());
        }
    }

    pub(crate) async fn is_group_member(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> anyhow::Result<bool> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::GET,
                    &format!(
                        "{}/Groups?filter=id eq \"{}\" and members eq \"{}\"",
                        &self.secret.endpoint, group_id, user_id
                    ),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .send()
                .await
                .context("Unable to send request to AWS SCIM (is_group_member)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            return res
                .error_for_status()
                .context("Error returned from server (is_group_member)")?
                .json::<ListResponse<Group>>()
                .await
                .context("Could not parse result from AWS SCIM (is_group_member)")
                .map(|d| !d.resources.is_empty());
        }
    }

    pub(crate) async fn add_group_member(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> anyhow::Result<()> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::PATCH,
                    &format!("{}/Groups/{}", &self.secret.endpoint, group_id),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .json(&serde_json::json!({
                   "schemas": [
                      "urn:ietf:params:scim:api:messages:2.0:PatchOp"
                   ],
                   "Operations": [{
                         "op": "add",
                         "path": "members",
                         "value": [{ "value": user_id }]
                    }]
                }))
                .send()
                .await
                .context("Unable to send request to AWS SCIM (add_group_member)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            let _ = res
                .error_for_status()
                .context("Error returned from server (add_group_member)")?;
            return Ok(());
        }
    }

    pub(crate) async fn remove_group_member(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> anyhow::Result<()> {
        use anyhow::Context;

        loop {
            let res = self
                .client
                .request(
                    reqwest::Method::PATCH,
                    &format!("{}/Groups/{}", &self.secret.endpoint, group_id),
                )
                .header(
                    "Authorization",
                    format!("Bearer {}", &self.secret.access_token),
                )
                .json(&serde_json::json!({
                   "schemas": [
                      "urn:ietf:params:scim:api:messages:2.0:PatchOp"
                   ],
                   "Operations": [{
                         "op": "remove",
                         "path": "members",
                         "value": [{ "value": user_id }]
                    }]
                }))
                .send()
                .await
                .context("Unable to send request to AWS SCIM (remove_group_member)")?;
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
            let _ = res
                .error_for_status()
                .context("Error returned from server (remove_group_member)")?;
            return Ok(());
        }
    }
}

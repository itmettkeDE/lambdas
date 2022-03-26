#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct Secret {
    pub(crate) region: String,
    pub(crate) id: String,
}

#[derive(Debug, Copy, Clone, serde::Deserialize)]
pub(crate) enum SyncStrategie {
    AllUsers,
    GroupMembersOnly,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Event {
    security_hub_google_creds: Option<Secret>,
    security_hub_scim_creds: Option<Secret>,
    google_api_query_for_users: Option<String>,
    google_api_query_for_groups: Option<String>,
    ignore_users_regexes: Option<Vec<String>>,
    include_users_regexes: Option<Vec<String>>,
    ignore_groups_regexes: Option<Vec<String>>,
    include_groups_regexes: Option<Vec<String>>,
    sync_strategie: Option<SyncStrategie>,
}

impl Event {
    pub(crate) fn get_security_hub_google_creds(
        &self,
    ) -> anyhow::Result<std::borrow::Cow<'_, Secret>> {
        use anyhow::{bail, Context};

        if let Some(ref v) = self.security_hub_google_creds {
            return Ok(std::borrow::Cow::Borrowed(v));
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_SH_GOOGLE_CREDS) {
            return serde_json::from_str(&env)
                .with_context(|| format!("{} is not a valid json object.", env));
        }
        bail!("Either the lambda event must contain an security_hub_google_creds object or the env variable {}, must be defined.", crate::ENV_VAR_SH_GOOGLE_CREDS)
    }

    pub(crate) fn get_security_hub_scim_creds(
        &self,
    ) -> anyhow::Result<std::borrow::Cow<'_, Secret>> {
        use anyhow::{bail, Context};

        if let Some(ref v) = self.security_hub_scim_creds {
            return Ok(std::borrow::Cow::Borrowed(v));
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_SH_SCIM_CREDS) {
            return serde_json::from_str(&env)
                .with_context(|| format!("{} is not a valid json object.", env));
        }
        bail!("Either the lambda event must contain an security_hub_scim_creds object or the env variable {}, must be defined.", crate::ENV_VAR_SH_SCIM_CREDS)
    }

    pub(crate) fn get_google_api_query_for_groups(&self) -> Option<std::borrow::Cow<'_, str>> {
        if let Some(ref v) = self.google_api_query_for_groups {
            return Some(std::borrow::Cow::Borrowed(v));
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_GOOGLE_API_QUERY_FOR_GROUPS) {
            return Some(env.into());
        }
        None
    }

    pub(crate) fn get_google_api_query_for_users(&self) -> Option<std::borrow::Cow<'_, str>> {
        if let Some(ref v) = self.google_api_query_for_users {
            return Some(std::borrow::Cow::Borrowed(v));
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_GOOGLE_API_QUERY_FOR_USERS) {
            return Some(env.into());
        }
        None
    }

    pub(crate) fn get_ignore_users_regexes(&self) -> anyhow::Result<Option<regex::RegexSet>> {
        use anyhow::Context;

        if let Some(ref v) = self.ignore_users_regexes {
            return regex::RegexSet::new(v)
                .with_context(|| {
                    format!(
                        "Unable to parse the following regex values from ignore_users_regexes: {:?}",
                        self.ignore_users_regexes
                    )
                })
                .map(Some);
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_IGNORE_USERS_REGEXES) {
            return regex::RegexSet::new(env.split(','))
                .with_context(|| {
                    format!(
                        "Unable to parse the following regex values from {}: {}",
                        crate::ENV_VAR_IGNORE_USERS_REGEXES,
                        env
                    )
                })
                .map(Some);
        }
        Ok(None)
    }

    pub(crate) fn get_include_users_regexes(&self) -> anyhow::Result<Option<regex::RegexSet>> {
        use anyhow::Context;

        if let Some(ref v) = self.include_users_regexes {
            return regex::RegexSet::new(v)
                .with_context(|| {
                    format!(
                        "Unable to parse the following regex values from include_users_regexes: {:?}",
                        self.include_users_regexes
                    )
                })
                .map(Some);
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_INCLUDE_USERS_REGEXES) {
            return regex::RegexSet::new(env.split(','))
                .with_context(|| {
                    format!(
                        "Unable to parse the following regex values from {}: {}",
                        crate::ENV_VAR_INCLUDE_USERS_REGEXES,
                        env
                    )
                })
                .map(Some);
        }
        Ok(None)
    }

    pub(crate) fn get_ignore_groups_regexes(&self) -> anyhow::Result<Option<regex::RegexSet>> {
        use anyhow::Context;

        if let Some(ref v) = self.ignore_groups_regexes {
            return regex::RegexSet::new(v)
                    .with_context(|| {
                        format!(
                            "Unable to parse the following regex values from ignore_groups_regexes: {:?}",
                            self.ignore_groups_regexes
                        )
                    })
                    .map(Some);
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_IGNORE_GROUPS_REGEXES) {
            return regex::RegexSet::new(env.split(','))
                .with_context(|| {
                    format!(
                        "Unable to parse the following regex values from {}: {}",
                        crate::ENV_VAR_IGNORE_GROUPS_REGEXES,
                        env
                    )
                })
                .map(Some);
        }
        Ok(None)
    }

    pub(crate) fn get_include_groups_regexes(&self) -> anyhow::Result<Option<regex::RegexSet>> {
        use anyhow::Context;

        if let Some(ref v) = self.include_groups_regexes {
            return regex::RegexSet::new(v)
                    .with_context(|| {
                        format!(
                            "Unable to parse the following regex values from include_groups_regexes: {:?}",
                            self.include_groups_regexes
                        )
                    })
                    .map(Some);
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_INCLUDE_GROUPS_REGEXES) {
            return regex::RegexSet::new(env.split(','))
                .with_context(|| {
                    format!(
                        "Unable to parse the following regex values from {}: {}",
                        crate::ENV_VAR_INCLUDE_GROUPS_REGEXES,
                        env
                    )
                })
                .map(Some);
        }
        Ok(None)
    }

    pub(crate) fn get_sync_strategie(&self) -> anyhow::Result<SyncStrategie> {
        use anyhow::Context;

        if let Some(v) = self.sync_strategie {
            return Ok(v);
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_SYNC_STRATEGIE) {
            return serde_json::from_str(&env)
                .with_context(|| format!("{} is not a valid sync strategie.", env));
        }
        Ok(SyncStrategie::GroupMembersOnly)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Event {
    pub bucket: Option<String>,
    pub prefix: Option<String>,
    pub invoke_time: Option<chrono::NaiveDateTime>,
    pub include_tags: Option<Vec<Tag>>,
    pub exclude_tags: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Tag {
    pub name: String,
    pub value: String,
}

impl Event {
    pub(crate) fn get_bucket(&self) -> anyhow::Result<std::borrow::Cow<'_, str>> {
        use anyhow::bail;

        if let Some(ref v) = self.bucket {
            return Ok(std::borrow::Cow::Borrowed(v));
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_BUCKET) {
            return Ok(env.into());
        }
        bail!("Either the lambda event must contain a bucket key or the env variable {}, must be defined.", crate::ENV_VAR_BUCKET)
    }

    pub(crate) fn get_prefix(&self) -> anyhow::Result<std::borrow::Cow<'_, str>> {
        use anyhow::bail;

        if let Some(ref v) = self.prefix {
            return Ok(std::borrow::Cow::Borrowed(v));
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_PREFIX) {
            return Ok(env.into());
        }
        bail!("Either the lambda event must contain a prefix key or the env variable {}, must be defined.", crate::ENV_VAR_PREFIX)
    }

    pub(crate) fn get_include_tags(&self) -> anyhow::Result<Option<std::borrow::Cow<'_, [Tag]>>> {
        use anyhow::Context;

        if let Some(ref v) = self.include_tags {
            return Ok(Some(std::borrow::Cow::Borrowed(v)));
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_INCLUDE_TAGS) {
            let tags = env
                .split(',')
                .map(|v| {
                    v.split_once('=').map(|(name, value)| Tag {
                        name: name.into(),
                        value: value.into(),
                    })
                })
                .collect::<Option<Vec<Tag>>>();
            return tags
                .with_context(|| {
                    format!(
                        "Unable to parse the following tag values from {}: {}",
                        crate::ENV_VAR_INCLUDE_TAGS,
                        env
                    )
                })
                .map(std::borrow::Cow::Owned)
                .map(Some);
        }
        Ok(None)
    }

    pub(crate) fn get_exclude_tags(&self) -> anyhow::Result<Option<std::borrow::Cow<'_, [Tag]>>> {
        use anyhow::Context;

        if let Some(ref v) = self.exclude_tags {
            return Ok(Some(std::borrow::Cow::Borrowed(v)));
        }
        if let Ok(env) = std::env::var(crate::ENV_VAR_EXCLUDE_TAGS) {
            let tags = env
                .split(',')
                .map(|v| {
                    v.split_once('=').map(|(name, value)| Tag {
                        name: name.into(),
                        value: value.into(),
                    })
                })
                .collect::<Option<Vec<Tag>>>();
            return tags
                .with_context(|| {
                    format!(
                        "Unable to parse the following tag values from {}: {}",
                        crate::ENV_VAR_EXCLUDE_TAGS,
                        env
                    )
                })
                .map(std::borrow::Cow::Owned)
                .map(Some);
        }
        Ok(None)
    }
}

mod scim;
mod smc;

pub(crate) use scim::{Group, Scim, ScimCreds, User, UserMail, UserName};

pub(crate) async fn get_secret_from_secret_manager<S: serde::de::DeserializeOwned>(
    secret: &super::event::Secret,
) -> anyhow::Result<S> {
    use anyhow::Context;
    use std::str::FromStr;

    let region = rusoto_core::Region::from_str(&secret.region)
        .with_context(|| format!("{} is not a valid AWS Region.", secret.region))?;
    let smc = smc::Smc::new(region);
    smc.get_secret_value_current(&secret.id).await
}

//! This tool syncs Users and Groups from Google Workspace to AWS SSO
//!
//! # Limitations
//!
//! AWS SCIM only returns 50 [Users](https://docs.aws.amazon.com/singlesignon/latest/developerguide/listusers.html)
//! or [Groups](https://docs.aws.amazon.com/singlesignon/latest/developerguide/listgroups.html).
//! This means:
//! * For Users: If you have more then 50 Users, the tool will still be able to remove
//! users added through Google Workspace, but it probably won't be able to remove manually
//! added users in AWS SSO.
//! * For Groups: If you have more then 50 Groups, the tool probably won't be able to
//! remove groups after they were deleted in Google Workspace. The reason for this is
//! that Google does not provide information about deleted groups. This also means, that
//! group membership will not be removed, as it is not possible to fetch all groups for
//! a User in AWS SCIM
//!
//! # Recommendations
//!
//! To combat these limitations and to get the best performance, adhere to the following
//! recommendations:
//! * Try to keep as few groups as possible (best is below 50) by using
//! `google_api_query_for_groups`, `ignore_groups_regexes` and/or
//! `include_groups_regexes`.
//! * Try to keep as few users as possible (best is below 50) by using
//! `google_api_query_for_users`, `ignore_users_regexes` and/or
//! `include_users_regexes`.
//! * Only sync users which are members of a group that is synced to AWS by using
//! the sync strategie `GroupMembersOnly`.
//!
//! # Setup
//!
//! * Enable `Admin SDK API` in the [Google Console](https://console.cloud.google.com/apis)<br>
//! (At the top of the Dashboard, there is a `Enable Apis and services` Button. Search for
//! `Admin SDK API` and click enable)
//! * Create a [Google Service User](https://developers.google.com/admin-sdk/directory/v1/guides/delegation)<br>
//! (Keep the credentials.json which is required at a later stage)
//! * Setup Domain-Wide Delegation Scopes:
//!   * https://www.googleapis.com/auth/admin.directory.group.readonly
//!   * https://www.googleapis.com/auth/admin.directory.group.member.readonly
//!   * https://www.googleapis.com/auth/admin.directory.user.readonly
//! * Enable Provisining in the AWS SSO Console <br>
//! (Keep Token and SCIM endpoint which are required at a later stage)
//! * Create a Secret in AWS Secret Manager with the following content:
//! ```json
//! {
//!   "endpoint": "<scim_endpoint>",
//!   "access_token": "<token>"
//! }
//! ```
//! * Create another Secret in AWS Secret Manager with the following content
//! ```json
//! {
//!   "mail": "<mail of a google admin user>",
//!   "credential_json": <credentials.json either as String or Object>
//! }
//! ```
//! * Create a lambda with the binary from this repository using runtime `provided.al2`
//! and anything as handler. (More Infos about paramters below)
//! * Create a CloudWatch Event to trigger the lambda regularly
//!
//! # Parameters
//!
//! The lambda function requires a few parameters to correctly work. You can define
//! them either with the Event that is send to the lambda, or via environment variables.
//!
//! ## Event
//!
//! ```json
//! {
//!     "security_hub_google_creds": {
//!         "region": "<region_of_secret>",
//!         "id": "<google_secret_name>"
//!     },
//!     "security_hub_scim_creds": {
//!         "region": "<region_of_secret>",
//!         "id": "<scim_secret_name>"
//!     },
//!     // Optional, remove if not required. Example: `email:aws-*`
//!     // Query send via Google API to filter users
//!     // More Infos at https://developers.google.com/admin-sdk/directory/v1/guides/search-users
//!     "google_api_query_for_users": "",
//!     // Optional, remove if not required. Example: `email:aws-*`
//!     // Query send via Google API to filter groups
//!     // More Infos at https://developers.google.com/admin-sdk/directory/v1/guides/search-groups
//!     "google_api_query_for_groups": "",
//!     // Optional, remove if not required. Example: `aws-.*@domain.org`
//!     // Ignores a user if one of the regexes matches. Matches on the primary_email
//!     "ignore_users_regexes": [],
//!     // Optional, remove if not required. Example: `aws-.*@domain.org`
//!     // Includes a user if one of the regexes matches. Matches on the primary_email
//!     "include_users_regexes": [],
//!     // Optional, remove if not required. Example: `aws-.*@domain.org`
//!     // Ignores a group if one of the regexes matches. Matches on the email
//!     "ignore_groups_regexes": [],
//!     // Optional, remove if not required. Example: `aws-.*@domain.org`
//!     // Includes a group if one of the regexes matches. Matches on the email
//!     "include_groups_regexes": [],
//!     // Optional, remove if not required. AllUsers | GroupMembersOnly (default)
//!     // Defines the sync strategie
//!     "sync_strategie": [],
//! }
//! ```
//!
//! ## Environment Variables
//! ```sh
//! SH_GOOGLE_CREDS="{\"region\": \"<region_of_secret>\",\"id\": \"<google_secret_name>\"}"
//! SH_SCIM_CREDS="{\"region\": \"<region_of_secret>\",\"id\": \"<scim_secret_name>\"}"
//! # Optional, skip if not required. Example: `email:aws-*`
//! # Query send via Google API to filter users
//! # More Infos at https://developers.google.com/admin-sdk/directory/v1/guides/search-users
//! GOOGLE_API_QUERY_FOR_USERS=""
//! # Optional, skip if not required. Example: `email:aws-*`
//! # Query send via Google API to filter groups
//! # More Infos at https://developers.google.com/admin-sdk/directory/v1/guides/search-groups
//! GOOGLE_API_QUERY_FOR_GROUPS=""
//! # Optional, skip if not required. Example: `aws-.*@domain.org`
//! # Ignores a user if one of the regexes matches. Matches on the primary_email
//! IGNORE_USERS_REGEXES=""
//! # Optional, skip if not required. Example: `aws-.*@domain.org`
//! # Includes a user if one of the regexes matches. Matches on the primary_email
//! INCLUDE_USERS_REGEXES=""
//! # Optional, skip if not required. Example: `aws-.*@domain.org`
//! # Ignores a group if one of the regexes matches. Matches on the email
//! IGNORE_GROUPS_REGEXES=""
//! # Optional, skip if not required. Example: `aws-.*@domain.org`
//! # Includes a group if one of the regexes matches. Matches on the email
//! INCLUDE_GROUPS_REGEXES=""
//! # Optional, skip if not required. AllUsers | GroupMembersOnly (default)
//! # Defines the sync strategie
//! SYNC_STRATEGIE=""
//! # Optional, skip if not required. off | error | warn | info (default) | debug | trace
//! # Defines the log level
//! LOG_LEVEL=""
//! ```
//!

#![warn(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    deprecated_in_future,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    indirect_structural_match,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_crate_level_docs,
    missing_debug_implementations,
    missing_docs,
    missing_doc_code_examples,
    non_ascii_idents,
    private_doc_tests,
    trivial_casts,
    trivial_numeric_casts,
    unaligned_references,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
#![warn(
    clippy::cargo,
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::perf,
    clippy::style
)]
#![allow(
    clippy::future_not_send,
    clippy::multiple_crate_versions,
    clippy::redundant_pub_crate,
    clippy::wildcard_dependencies
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod aws;
mod event;
mod google;
mod sync;

#[cfg(feature = "test")]
const TEST_DATA: &str = include_str!("../test.json");

const ENV_VAR_LOG_LEVEL: &str = "LOG_LEVEL";
pub(crate) const ENV_VAR_SH_GOOGLE_CREDS: &str = "SH_GOOGLE_CREDS";
pub(crate) const ENV_VAR_SH_SCIM_CREDS: &str = "SH_SCIM_CREDS";
pub(crate) const ENV_VAR_GOOGLE_API_QUERY_FOR_USERS: &str = "GOOGLE_API_QUERY_FOR_USERS";
pub(crate) const ENV_VAR_GOOGLE_API_QUERY_FOR_GROUPS: &str = "GOOGLE_API_QUERY_FOR_GROUPS";
pub(crate) const ENV_VAR_IGNORE_USERS_REGEXES: &str = "IGNORE_USERS_REGEXES";
pub(crate) const ENV_VAR_INCLUDE_USERS_REGEXES: &str = "INCLUDE_USERS_REGEXES";
pub(crate) const ENV_VAR_IGNORE_GROUPS_REGEXES: &str = "IGNORE_GROUPS_REGEXES";
pub(crate) const ENV_VAR_INCLUDE_GROUPS_REGEXES: &str = "INCLUDE_GROUPS_REGEXES";
pub(crate) const ENV_VAR_SYNC_STRATEGIE: &str = "SYNC_STRATEGIE";

struct Runner;

#[async_trait::async_trait]
impl<'a> lambda_runtime_types::Runner<'a, (), event::Event, ()> for Runner {
    async fn run(
        _shared: &'a (),
        event: lambda_runtime_types::LambdaEvent<'a, event::Event>,
    ) -> anyhow::Result<()> {
        let security_hub_google_creds: google::AdminCreds = aws::get_secret_from_secret_manager(
            event.event.get_security_hub_google_creds()?.as_ref(),
        )
        .await?;
        let security_hub_scim_creds: aws::ScimCreds = aws::get_secret_from_secret_manager(
            event.event.get_security_hub_scim_creds()?.as_ref(),
        )
        .await?;

        let scim = aws::Scim::new(&security_hub_scim_creds);
        let gadmin = google::Admin::new(&security_hub_google_creds).await?;

        let mut sync_op = sync::SyncOp::new(&event.event, &scim, &gadmin).await?;
        sync_op.sync_groups().await?;
        sync_op
            .sync_users(event.event.get_sync_strategie()?)
            .await?;
        sync_op.sync_associations().await?;
        Ok(())
    }

    async fn setup(_region: &'a str) -> anyhow::Result<()> {
        use anyhow::Context;
        use std::str::FromStr;

        let log_level = std::env::var(ENV_VAR_LOG_LEVEL);
        let log_level = log_level.as_ref().map(AsRef::as_ref).unwrap_or("info");
        let log_level = log::LevelFilter::from_str(log_level)
            .with_context(|| format!("Invalid log_level: {}", log_level))?;
        simple_logger::SimpleLogger::new()
            .with_level(log_level)
            .init()
            .expect("Unable to setup logging");
        Ok(())
    }
}

/// Entrypoint for the lambda
pub fn main() -> anyhow::Result<()> {
    #[cfg(not(feature = "test"))]
    {
        lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
    }
    #[cfg(feature = "test")]
    {
        lambda_runtime_types::exec_test::<_, _, Runner, _>(TEST_DATA)
    }
}

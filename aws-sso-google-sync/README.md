# aws-sso-google-sync

This tool syncs Users and Groups from Google Workspace to AWS SSO

## Limitations

AWS SCIM only returns 50 [Users](https://docs.aws.amazon.com/singlesignon/latest/developerguide/listusers.html)
or [Groups](https://docs.aws.amazon.com/singlesignon/latest/developerguide/listgroups.html).
This means:
* For Users: If you have more then 50 Users, the tool will still be able to remove
users added through Google Workspace, but it probably won't be able to remove manually
added users in AWS SSO.
* For Groups: If you have more then 50 Groups, the tool probably won't be able to
remove groups after they were deleted in Google Workspace. The reason for this is
that Google does not provide information about deleted groups. This also means, that
group membership will not be removed, as it is not possible to fetch all groups for
a User in AWS SCIM

## Recommendations

To combat these limitations and to get the best performance, adhere to the following
recommendations:
* Try to keep as few groups as possible (best is below 50) by using
`google_api_query_for_groups`, `ignore_groups_regexes` and/or
`include_groups_regexes`.
* Try to keep as few users as possible (best is below 50) by using
`google_api_query_for_users`, `ignore_users_regexes` and/or
`include_users_regexes`.
* Only sync users which are members of a group that is synced to AWS by using
the sync strategie `GroupMembersOnly`.

## Setup

* Enable `Admin SDK API` in the [Google Console](https://console.cloud.google.com/apis)<br>
(At the top of the Dashboard, there is a `Enable Apis and services` Button. Search for
`Admin SDK API` and click enable)
* Create a [Google Service User](https://developers.google.com/admin-sdk/directory/v1/guides/delegation)<br>
(Keep the credentials.json which is required at a later stage)
* Setup Domain-Wide Delegation Scopes:
  * https://www.googleapis.com/auth/admin.directory.group.readonly
  * https://www.googleapis.com/auth/admin.directory.group.member.readonly
  * https://www.googleapis.com/auth/admin.directory.user.readonly
* Enable Provisining in the AWS SSO Console <br>
(Keep Token and SCIM endpoint which are required at a later stage)
* Create a Secret in AWS Secret Manager with the following content:
```json
{
  "endpoint": "<scim_endpoint>",
  "access_token": "<token>"
}
```
* Create another Secret in AWS Secret Manager with the following content
```json
{
  "mail": "<mail of a google admin user>",
  "credential_json": <credentials.json either as String or Object>
}
```
* Create a lambda with the binary from this repository using runtime `provided.al2`
and anything as handler. (More Infos about paramters below)
* Create a CloudWatch Event to trigger the lambda regularly

## Parameters

The lambda function requires a few parameters to correctly work. You can define
them either with the Event that is send to the lambda, or via environment variables.

### Event

```json
{
    "security_hub_google_creds": {
        "region": "<region_of_secret>",
        "id": "<google_secret_name>"
    },
    "security_hub_scim_creds": {
        "region": "<region_of_secret>",
        "id": "<scim_secret_name>"
    },
    // Optional, remove if not required. Example: `email:aws-*`
    // Query send via Google API to filter users
    // More Infos at https://developers.google.com/admin-sdk/directory/v1/guides/search-users
    "google_api_query_for_users": "",
    // Optional, remove if not required. Example: `email:aws-*`
    // Query send via Google API to filter groups
    // More Infos at https://developers.google.com/admin-sdk/directory/v1/guides/search-groups
    "google_api_query_for_groups": "",
    // Optional, remove if not required. Example: `aws-.*@domain.org`
    // Ignores a user if one of the regexes matches. Matches on the primary_email
    "ignore_users_regexes": [],
    // Optional, remove if not required. Example: `aws-.*@domain.org`
    // Includes a user if one of the regexes matches. Matches on the primary_email
    "include_users_regexes": [],
    // Optional, remove if not required. Example: `aws-.*@domain.org`
    // Ignores a group if one of the regexes matches. Matches on the email
    "ignore_groups_regexes": [],
    // Optional, remove if not required. Example: `aws-.*@domain.org`
    // Includes a group if one of the regexes matches. Matches on the email
    "include_groups_regexes": [],
    // Optional, remove if not required. AllUsers | GroupMembersOnly (default)
    // Defines the sync strategie
    "sync_strategie": [],
}
```

### Environment Variables
```sh
SH_GOOGLE_CREDS="{\"region\": \"<region_of_secret>\",\"id\": \"<google_secret_name>\"}"
SH_SCIM_CREDS="{\"region\": \"<region_of_secret>\",\"id\": \"<scim_secret_name>\"}"
# Optional, skip if not required. Example: `email:aws-*`
# Query send via Google API to filter users
# More Infos at https://developers.google.com/admin-sdk/directory/v1/guides/search-users
GOOGLE_API_QUERY_FOR_USERS=""
# Optional, skip if not required. Example: `email:aws-*`
# Query send via Google API to filter groups
# More Infos at https://developers.google.com/admin-sdk/directory/v1/guides/search-groups
GOOGLE_API_QUERY_FOR_GROUPS=""
# Optional, skip if not required. Example: `aws-.*@domain.org`
# Ignores a user if one of the regexes matches. Matches on the primary_email
IGNORE_USERS_REGEXES=""
# Optional, skip if not required. Example: `aws-.*@domain.org`
# Includes a user if one of the regexes matches. Matches on the primary_email
INCLUDE_USERS_REGEXES=""
# Optional, skip if not required. Example: `aws-.*@domain.org`
# Ignores a group if one of the regexes matches. Matches on the email
IGNORE_GROUPS_REGEXES=""
# Optional, skip if not required. Example: `aws-.*@domain.org`
# Includes a group if one of the regexes matches. Matches on the email
INCLUDE_GROUPS_REGEXES=""
# Optional, skip if not required. AllUsers | GroupMembersOnly (default)
# Defines the sync strategie
SYNC_STRATEGIE=""
# Optional, skip if not required. off | error | warn | info (default) | debug | trace
# Defines the log level
LOG_LEVEL=""
```


License: MIT OR Apache-2.0

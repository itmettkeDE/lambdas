#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum ApiGatewayEvent {
    ApiGatewayEventV1(ApiGatewayEventV1),
    ApiGatewayEventV2(ApiGatewayEventV2),
}

impl ApiGatewayEvent {
    pub fn headers(&self) -> std::borrow::Cow<'_, http::HeaderMap> {
        let headers = match self {
            Self::ApiGatewayEventV1(event) => event
                .multi_value_headers
                .as_ref()
                .or(event.headers.as_ref()),
            Self::ApiGatewayEventV2(event) => event.headers.as_ref(),
        };
        headers
            .map(std::ops::Deref::deref)
            .map(std::borrow::Cow::Borrowed)
            .unwrap_or_else(|| std::borrow::Cow::Owned(http::HeaderMap::default()))
    }

    pub fn body(&self) -> std::borrow::Cow<'_, str> {
        let body = match self {
            Self::ApiGatewayEventV1(event) => &event.body,
            Self::ApiGatewayEventV2(event) => &event.body,
        };
        body.as_ref()
            .map(AsRef::as_ref)
            .map(std::borrow::Cow::Borrowed)
            .unwrap_or_else(|| std::borrow::Cow::Borrowed(""))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV1 {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub http_method: Option<String>,
    #[serde(default)]
    pub headers: Option<HeaderMap>,
    #[serde(default)]
    pub multi_value_headers: Option<HeaderMap>,
    #[serde(default, deserialize_with = "query_map::deserialize_empty")]
    pub query_string_parameters: query_map::QueryMap,
    pub multi_value_query_string_parameters: Option<query_map::QueryMap>,
    #[serde(default)]
    pub request_context: Option<ApiGatewayEventV1RequestContext>,
    #[serde(default)]
    pub path_parameters: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub stage_variables: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub is_base64_encoded: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV1RequestContext {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub apiid: Option<String>,
    #[serde(default)]
    pub authorizer: Option<ApiGatewayEventV1Authorizer>,
    #[serde(default)]
    pub domain_name: Option<String>,
    #[serde(default)]
    pub domain_prefix: Option<String>,
    #[serde(default)]
    pub extended_request_id: Option<String>,
    #[serde(default)]
    pub http_method: Option<String>,
    #[serde(default)]
    pub identity: Option<ApiGatewayEventV1Identity>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub request_time: Option<String>,
    #[serde(default)]
    pub request_time_epoch: Option<i64>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub resource_path: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV1Authorizer {
    #[serde(default)]
    pub claims: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV1Identity {
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub caller: Option<String>,
    #[serde(default)]
    pub cognito_authentication_provider: Option<String>,
    #[serde(default)]
    pub cognito_authentication_type: Option<String>,
    #[serde(default)]
    pub cognito_identity_id: Option<String>,
    #[serde(default)]
    pub cognito_identity_pool_id: Option<String>,
    #[serde(default)]
    pub principal_org_id: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub user_arn: Option<String>,
    #[serde(default)]
    pub client_cert: Option<ApiGatewayEventV1ClientCert>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV1ClientCert {
    #[serde(default)]
    pub client_cert_pem: Option<String>,
    #[serde(default)]
    pub subject_dn: Option<String>,
    #[serde(default)]
    pub issuer_dn: Option<String>,
    #[serde(default)]
    pub serial_number: Option<String>,
    #[serde(default)]
    pub validity: Option<ApiGatewayEventV1Validity>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV1Validity {
    #[serde(default)]
    pub not_before: Option<String>,
    #[serde(default)]
    pub not_after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV2<Auth = serde_json::Value> {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default)]
    pub raw_path: Option<String>,
    #[serde(default)]
    pub raw_query_string: Option<String>,
    #[serde(default)]
    pub cookies: Option<Vec<String>>,
    #[serde(default)]
    pub headers: Option<HeaderMap>,
    #[serde(default, deserialize_with = "query_map::deserialize_empty")]
    pub query_string_parameters: query_map::QueryMap,
    #[serde(default)]
    pub request_context: Option<ApiGatewayEventV2RequestContext<Auth>>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub path_parameters: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub is_base64_encoded: Option<bool>,
    #[serde(default)]
    pub stage_variables: Option<std::collections::HashMap<String, String>>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderMap(#[serde(default, with = "http_serde::header_map")] pub http::HeaderMap);

impl AsRef<http::HeaderMap> for HeaderMap {
    fn as_ref(&self) -> &http::HeaderMap {
        &self.0
    }
}

impl AsMut<http::HeaderMap> for HeaderMap {
    fn as_mut(&mut self) -> &mut http::HeaderMap {
        &mut self.0
    }
}

impl std::ops::Deref for HeaderMap {
    type Target = http::HeaderMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for HeaderMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV2RequestContext<Auth> {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub apiid: Option<String>,
    #[serde(default)]
    pub authentication: Option<ApiGatewayEventV2Authentication>,
    #[serde(default)]
    pub authorizer: Option<ApiGatewayEventV2Authorizer<Auth>>,
    #[serde(default)]
    pub domain_name: Option<String>,
    #[serde(default)]
    pub domain_prefix: Option<String>,
    #[serde(default)]
    pub http: Option<ApiGatewayEventV2Http>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub time_epoch: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV2Authorizer<Auth> {
    #[serde(default)]
    pub jwt: Option<ApiGatewayEventV2Jwt>,
    #[serde(default)]
    pub lambda: Option<std::collections::HashMap<String, Auth>>,
    #[serde(default)]
    pub iam: Option<ApiGatewayEventV2Iam>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV2Jwt {
    #[serde(default)]
    pub claims: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV2Iam {
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub caller_id: Option<String>,
    #[serde(default)]
    pub cognito_identity: Option<ApiGatewayEventV2CognitoIdentity>,
    #[serde(default)]
    pub principal_org_id: Option<String>,
    #[serde(default)]
    pub user_arn: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV2CognitoIdentity {
    #[serde(default)]
    pub amr: Option<Vec<String>>,
    #[serde(default)]
    pub identity_id: Option<String>,
    #[serde(default)]
    pub identity_pool_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV2Http {
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV2Authentication {
    #[serde(default)]
    pub client_cert: Option<ApiGatewayEventV2ClientCert>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV2ClientCert {
    #[serde(default)]
    pub client_cert_pem: Option<String>,
    #[serde(default)]
    pub subject_dn: Option<String>,
    #[serde(default)]
    pub issuer_dn: Option<String>,
    #[serde(default)]
    pub serial_number: Option<String>,
    #[serde(default)]
    pub validity: Option<ApiGatewayEventV2Validity>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayEventV2Validity {
    #[serde(default)]
    pub not_before: Option<String>,
    #[serde(default)]
    pub not_after: Option<String>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn deserialize_v2_empty() {
        let json_str = include_str!("../tests/event_empty.json");
        let _: super::ApiGatewayEvent =
            serde_json::from_str(json_str).expect("Unable to parse json");
    }

    #[test]
    fn deserialize_v1_01() {
        let json_str = include_str!("../tests/event_v1_01.json");
        let _: super::ApiGatewayEvent =
            serde_json::from_str(json_str).expect("Unable to parse json");
    }

    #[test]
    fn deserialize_v1_02() {
        let json_str = include_str!("../tests/event_v1_02.json");
        let _: super::ApiGatewayEvent =
            serde_json::from_str(json_str).expect("Unable to parse json");
    }

    #[test]
    fn deserialize_v2_01() {
        let json_str = include_str!("../tests/event_v2_01.json");
        let _: super::ApiGatewayEvent =
            serde_json::from_str(json_str).expect("Unable to parse json");
    }

    #[test]
    fn deserialize_v2_02() {
        let json_str = include_str!("../tests/event_v2_02.json");
        let _: super::ApiGatewayEvent =
            serde_json::from_str(json_str).expect("Unable to parse json");
    }

    #[test]
    fn deserialize_v2_03() {
        let json_str = include_str!("../tests/event_v2_03.json");
        let _: super::ApiGatewayEvent =
            serde_json::from_str(json_str).expect("Unable to parse json");
    }
}

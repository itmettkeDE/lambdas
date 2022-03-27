#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(serialize_with = "serialize_status_code")]
    pub status_code: http::StatusCode,
    pub headers: Option<HeaderMapSingle>,
    pub multi_value_headers: Option<HeaderMapMulti>,
    pub body: Option<Body>,
    pub is_base64_encoded: Option<bool>,
}

impl Response {
    pub const fn simple(status_code: http::StatusCode) -> Self {
        Self {
            status_code,
            headers: None,
            multi_value_headers: None,
            body: None,
            is_base64_encoded: None,
        }
    }
}

fn serialize_status_code<S>(
    status_code: &http::StatusCode,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    use serde::ser::Serialize;

    status_code.as_u16().serialize(serializer)
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderMapMulti(#[serde(with = "http_serde::header_map")] pub http::HeaderMap);

impl From<http::HeaderMap> for HeaderMapMulti {
    fn from(h: http::HeaderMap) -> Self {
        Self(h)
    }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderMapSingle(#[serde(serialize_with = "serialize_headers")] pub http::HeaderMap);

impl From<http::HeaderMap> for HeaderMapSingle {
    fn from(h: http::HeaderMap) -> Self {
        Self(h)
    }
}

fn serialize_headers<S>(headers: &http::HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    use serde::ser::{Error, SerializeMap};

    let mut map = serializer.serialize_map(Some(headers.keys_len()))?;
    for key in headers.keys() {
        let map_value = headers[key].to_str().map_err(S::Error::custom)?;
        map.serialize_entry(key.as_str(), map_value)?;
    }
    map.end()
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Body {}

// SPDX-License-Identifier: GPL-3.0-only

use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::http::constants::{APPLICATION_FORM, APPLICATION_JSON, EMPTY_JSON_OBJECT};
use crate::http::error::{Error, Result, body_preview};

/// Which site a [`Request`]'s path is resolved against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Site {
    /// The metering portal ([`crate::http::ClientConfig::base_url`]).
    #[default]
    Portal,
    /// Provident's corporate site, which publishes rates and fees
    /// ([`crate::http::ClientConfig::rates_base_url`]).
    Rates,
}

/// Transport-agnostic description of a single request, executed by [`crate::http::Client`].
///
/// Built with [`Request::builder`] or a method shortcut ([`Request::get`],
/// [`Request::page_method`], …), then `.build()`.
#[derive(Debug, Clone, bon::Builder)]
#[builder(derive(Debug, Clone))]
pub struct Request {
    /// HTTP method to send.
    #[builder(start_fn)]
    pub(crate) method: Method,

    /// Relative to the base URL of [`Request::site`]; a leading `/` is ignored so it never
    /// discards the base path.
    #[builder(start_fn, into)]
    pub(crate) path: String,

    /// Per-request headers; override client defaults. Set via [`RequestBuilder::header`].
    #[builder(field)]
    pub(crate) headers: HeaderMap,

    /// Encoded query string. Set via [`RequestBuilder::query`].
    #[builder(field)]
    pub(crate) query: Option<String>,

    /// Serialized body. Set via [`RequestBuilder::json`] or [`RequestBuilder::form`].
    #[builder(field)]
    pub(crate) body: Option<Vec<u8>>,

    /// Site whose base URL `path` is joined to. Set via [`RequestBuilder::site`].
    #[builder(field)]
    pub(crate) site: Site,

    /// Decode the response from an ASP.NET `{"d": ...}` envelope. Set by [`Request::page_method`].
    #[builder(field)]
    pub(crate) unwrap_d: bool,
}

impl Request {
    /// Starts a `GET` request to `path`.
    pub fn get(path: impl Into<String>) -> RequestBuilder {
        Self::builder(Method::GET, path)
    }

    /// Starts a `POST` request to `path`.
    pub fn post(path: impl Into<String>) -> RequestBuilder {
        Self::builder(Method::POST, path)
    }

    /// Starts a `PUT` request to `path`.
    pub fn put(path: impl Into<String>) -> RequestBuilder {
        Self::builder(Method::PUT, path)
    }

    /// Starts a `PATCH` request to `path`.
    pub fn patch(path: impl Into<String>) -> RequestBuilder {
        Self::builder(Method::PATCH, path)
    }

    /// Starts a `DELETE` request to `path`.
    pub fn delete(path: impl Into<String>) -> RequestBuilder {
        Self::builder(Method::DELETE, path)
    }

    /// ASP.NET WebForms page method (`Page.aspx/Method`): always sends
    /// `Content-Type: application/json` (without it ASP.NET serves the page's HTML, even for
    /// GET), and removes the response's `{"d": ...}` wrapper before decoding.
    ///
    /// `method` must match the server's `[WebMethod]` (`GET` only for `UseHttpGet = true`;
    /// otherwise ASP.NET answers 500). Non-GET requests get an empty `{}` body; call
    /// [`RequestBuilder::json`] to send arguments, whose names must match the C# parameters.
    pub fn page_method(method: Method, path: impl Into<String>) -> RequestBuilder {
        let body = (method != Method::GET).then(|| EMPTY_JSON_OBJECT.to_vec());
        let mut builder = Self::builder(method, path);
        builder.body = body;
        builder
            .headers
            .insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_JSON));
        builder.unwrap_d = true;
        builder
    }

    /// HTTP method of this request.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Path relative to the client's base URL, as given (a leading `/` is not stripped here).
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Site whose base URL the path is resolved against.
    pub fn site(&self) -> Site {
        self.site
    }

    pub(crate) fn url(&self, base: &Url) -> Result<Url> {
        let mut url = base.join(self.path.trim_start_matches('/'))?;
        if let Some(query) = &self.query {
            url.set_query(Some(query));
        }
        Ok(url)
    }
}

impl<S: request_builder::State> RequestBuilder<S> {
    /// Sets the query string from any `Serialize` value (struct, map, slice of pairs).
    pub fn query<Q: Serialize + ?Sized>(mut self, query: &Q) -> Result<Self> {
        let encoded = serde_urlencoded::to_string(query)?;
        self.query = (!encoded.is_empty()).then_some(encoded);
        Ok(self)
    }

    /// Sets a JSON body and `Content-Type: application/json`.
    pub fn json<B: Serialize + ?Sized>(mut self, body: &B) -> Result<Self> {
        self.body = Some(serde_json::to_vec(body).map_err(Error::Serialize)?);
        self.headers
            .insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_JSON));
        Ok(self)
    }

    /// Sets a form-encoded body and `Content-Type: application/x-www-form-urlencoded`.
    pub fn form<B: Serialize + ?Sized>(mut self, body: &B) -> Result<Self> {
        self.body = Some(serde_urlencoded::to_string(body)?.into_bytes());
        self.headers
            .insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_FORM));
        Ok(self)
    }

    /// Resolves the path against `site`'s base URL instead of the portal's.
    pub fn site(mut self, site: Site) -> Self {
        self.site = site;
        self
    }

    /// Adds (or replaces) a header for this request only; overrides client defaults.
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(name, value);
        self
    }
}

/// A typed API operation: knows how to build its [`Request`] and what it returns.
///
/// API modules implement this per endpoint, then call `client.execute(&endpoint)`.
pub trait Endpoint {
    /// Deserialized response body. Use `()` for endpoints with no/empty body.
    type Response: DeserializeOwned;

    /// Builds the [`Request`] for this operation; fails if the query or body cannot be encoded.
    fn request(&self) -> Result<Request>;
}

/// ASP.NET page method response envelope (`{"d": ...}`).
#[derive(serde::Deserialize)]
struct Wrapped<T> {
    d: T,
}

/// Maps a raw response to `T`: non-2xx → [`Error::Status`]; empty body decodes as JSON `null`
/// so `()` and `Option<T>` work for bodiless responses. With `unwrap_d`, the body must be a
/// `{"d": ...}` envelope (void page methods return `{"d":null}`) and `T` is decoded from `d`.
pub(crate) fn decode<T: DeserializeOwned>(
    status: StatusCode,
    bytes: &[u8],
    unwrap_d: bool,
) -> Result<T> {
    if !status.is_success() {
        return Err(Error::Status {
            status,
            body: body_preview(bytes),
        });
    }
    let json: &[u8] = if bytes.is_empty() { b"null" } else { bytes };
    let decoded = if unwrap_d {
        serde_json::from_slice::<Wrapped<T>>(json).map(|w| w.d)
    } else {
        serde_json::from_slice(json)
    };
    decoded.map_err(|source| Error::Deserialize {
        status,
        body: body_preview(bytes),
        source,
    })
}

/// Maps a raw response to its body as text (lossy UTF-8): non-2xx → [`Error::Status`].
pub(crate) fn text(status: StatusCode, bytes: &[u8]) -> Result<String> {
    if !status.is_success() {
        return Err(Error::Status {
            status,
            body: body_preview(bytes),
        });
    }
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        crate::http::ClientConfig::builder()
            .base_url("https://api.example.com/v1")
            .unwrap()
            .build()
            .base_url()
            .clone()
    }

    #[test]
    fn url_keeps_base_path_even_with_leading_slash() {
        let url = Request::get("/users/42").build().url(&base()).unwrap();
        assert_eq!(url.as_str(), "https://api.example.com/v1/users/42");
    }

    #[test]
    fn url_encodes_query() {
        let url = Request::get("search")
            .query(&[("q", "a b&c"), ("page", "2")])
            .unwrap()
            .build()
            .url(&base())
            .unwrap();
        assert_eq!(
            url.as_str(),
            "https://api.example.com/v1/search?q=a+b%26c&page=2"
        );
    }

    #[test]
    fn decode_empty_body_as_unit_and_none() {
        decode::<()>(StatusCode::NO_CONTENT, b"", false).unwrap();
        assert_eq!(
            decode::<Option<u32>>(StatusCode::OK, b"", false).unwrap(),
            None
        );
    }

    #[test]
    fn decode_non_success_is_status_error_with_body() {
        let err = decode::<()>(StatusCode::NOT_FOUND, b"{\"error\":\"nope\"}", true).unwrap_err();
        assert!(
            matches!(&err, Error::Status { status, body } if *status == StatusCode::NOT_FOUND && body.contains("nope"))
        );
        assert_eq!(err.status(), Some(StatusCode::NOT_FOUND));
    }

    #[test]
    fn decode_invalid_json_is_deserialize_error() {
        let err = decode::<u32>(StatusCode::OK, b"not json", false).unwrap_err();
        assert!(matches!(err, Error::Deserialize { .. }));
    }

    #[test]
    fn decode_unwraps_d_envelope() {
        assert_eq!(
            decode::<Vec<u32>>(StatusCode::OK, b"{\"d\":[1,2]}", true).unwrap(),
            [1, 2]
        );
        // `void` page methods answer `{"d":null}`.
        decode::<()>(StatusCode::OK, b"{\"d\":null}", true).unwrap();
    }

    #[test]
    fn decode_missing_d_envelope_is_deserialize_error() {
        let err = decode::<Vec<u32>>(StatusCode::OK, b"[1,2]", true).unwrap_err();
        assert!(matches!(err, Error::Deserialize { .. }));
    }
}

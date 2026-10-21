// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use reqwest::header::{ACCEPT, HeaderMap, HeaderName, HeaderValue};
use url::Url;

use crate::http::constants::{
    APPLICATION_JSON, DEFAULT_BASE_URL, DEFAULT_CONNECT_TIMEOUT, DEFAULT_RATES_BASE_URL,
    DEFAULT_TIMEOUT, USER_AGENT,
};
use crate::http::error::{Error, Result};
use crate::http::request::Site;

/// Settings for [`crate::http::Client`], built via [`ClientConfig::builder`]. Unset members fall back
/// to [`crate::http::constants`]; [`ClientConfig::default`] equals `ClientConfig::builder().build()`.
#[derive(Debug, Clone, bon::Builder)]
#[builder(derive(Debug, Clone))]
pub struct ClientConfig {
    /// Headers sent with every request; `Accept: application/json` by default.
    /// Add more (e.g. authorization) with [`ClientConfigBuilder::header`].
    #[builder(field = default_headers())]
    pub(crate) headers: HeaderMap,

    /// Endpoint paths are resolved relative to this,
    /// so `https://api.example.com/v1` + `users` → `https://api.example.com/v1/users`.
    #[builder(
        default = parse_base_url(DEFAULT_BASE_URL).expect("DEFAULT_BASE_URL must be a valid base URL"),
        with = |base_url: &str| -> Result<_> { parse_base_url(base_url) },
    )]
    pub(crate) base_url: Url,

    /// Base URL of Provident's corporate site (published rates and fees), for
    /// [`Site::Rates`] requests. Same joining rules as `base_url`.
    #[builder(
        default = parse_base_url(DEFAULT_RATES_BASE_URL).expect("DEFAULT_RATES_BASE_URL must be a valid base URL"),
        with = |rates_base_url: &str| -> Result<_> { parse_base_url(rates_base_url) },
    )]
    pub(crate) rates_base_url: Url,

    /// Total per-request timeout, from start of connect until the body is read.
    #[builder(default = DEFAULT_TIMEOUT)]
    pub(crate) timeout: Duration,

    /// Timeout for establishing the connection only.
    #[builder(default = DEFAULT_CONNECT_TIMEOUT)]
    pub(crate) connect_timeout: Duration,

    /// `User-Agent` header value sent with every request.
    #[builder(into, default = USER_AGENT.to_owned())]
    pub(crate) user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl ClientConfig {
    /// Base URL that endpoint paths are resolved against; always ends with `/`.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Base URL of the rates site; always ends with `/`.
    pub fn rates_base_url(&self) -> &Url {
        &self.rates_base_url
    }

    /// Base URL that paths of `site` are resolved against.
    pub(crate) fn site_url(&self, site: Site) -> &Url {
        match site {
            Site::Portal => &self.base_url,
            Site::Rates => &self.rates_base_url,
        }
    }
}

impl<S: client_config_builder::State> ClientConfigBuilder<S> {
    /// Adds (or replaces) a header sent with every request, e.g. authorization.
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(name, value);
        self
    }
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static(APPLICATION_JSON));
    headers
}

fn parse_base_url(base_url: &str) -> Result<Url> {
    let mut url = Url::parse(base_url)?;
    if url.cannot_be_a_base() {
        return Err(Error::UrlNotBase(url));
    }
    // `Url::join` drops the last segment unless the path ends with '/'.
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_override_rejects_non_base_url() {
        let err = ClientConfig::builder()
            .base_url("mailto:x@example.com")
            .unwrap_err();
        assert!(matches!(err, Error::UrlNotBase(_)));
    }

    #[test]
    fn header_keeps_default_accept() {
        let config = ClientConfig::builder()
            .header(
                reqwest::header::AUTHORIZATION,
                HeaderValue::from_static("x"),
            )
            .build();
        assert_eq!(config.headers[ACCEPT], APPLICATION_JSON);
        assert_eq!(config.headers[reqwest::header::AUTHORIZATION], "x");
    }
}

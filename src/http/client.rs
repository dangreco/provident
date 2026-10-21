// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::http::config::ClientConfig;
use crate::http::error::Result;
use crate::http::request::{Endpoint, Request, decode, text};

#[cfg(feature = "async")]
use reqwest::Client as Inner;
#[cfg(feature = "blocking")]
use reqwest::blocking::Client as Inner;

/// JSON HTTP client. Async with feature `async`, blocking with feature `blocking`.
/// Cheap to clone; clones share the connection pool.
///
/// With `blocking`, must not be created or used inside an async runtime (reqwest panics).
#[derive(Debug, Clone)]
pub struct Client {
    inner: Inner,
    config: Arc<ClientConfig>,
}

impl Client {
    /// Builds a client from `config`, with a cookie store so session cookies persist across
    /// requests and clones.
    pub fn new(config: ClientConfig) -> Result<Self> {
        let inner = Inner::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .user_agent(&config.user_agent)
            // Persists session cookies (e.g. ASP.NET forms auth) across requests and clones.
            .cookie_store(true)
            .default_headers(config.headers.clone())
            .build()?;
        Ok(Self {
            inner,
            config: Arc::new(config),
        })
    }

    /// Configuration this client was built with.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Sends `request` and decodes the JSON response into `T`.
    #[cfg(feature = "async")]
    pub async fn send<T: DeserializeOwned>(&self, request: Request) -> Result<T> {
        let unwrap_d = request.unwrap_d;
        let response = self.build(request)?.send().await?;
        let status = response.status();
        let bytes = response.bytes().await?;
        decode(status, &bytes, unwrap_d)
    }

    /// Sends `request` and decodes the JSON response into `T`.
    #[cfg(feature = "blocking")]
    pub fn send<T: DeserializeOwned>(&self, request: Request) -> Result<T> {
        let unwrap_d = request.unwrap_d;
        let response = self.build(request)?.send()?;
        let status = response.status();
        let bytes = response.bytes()?;
        decode(status, &bytes, unwrap_d)
    }

    /// Sends `request` and returns the body as text (e.g. an HTML page); non-2xx →
    /// [`crate::http::Error::Status`].
    #[cfg(feature = "async")]
    pub async fn send_text(&self, request: Request) -> Result<String> {
        let response = self.build(request)?.send().await?;
        let status = response.status();
        let bytes = response.bytes().await?;
        text(status, &bytes)
    }

    /// Sends `request` and returns the body as text (e.g. an HTML page); non-2xx →
    /// [`crate::http::Error::Status`].
    #[cfg(feature = "blocking")]
    pub fn send_text(&self, request: Request) -> Result<String> {
        let response = self.build(request)?.send()?;
        let status = response.status();
        let bytes = response.bytes()?;
        text(status, &bytes)
    }

    /// Builds the endpoint's [`Request`], sends it, and decodes the response as
    /// [`Endpoint::Response`].
    #[cfg(feature = "async")]
    pub async fn execute<E: Endpoint>(&self, endpoint: &E) -> Result<E::Response> {
        self.send(endpoint.request()?).await
    }

    /// Builds the endpoint's [`Request`], sends it, and decodes the response as
    /// [`Endpoint::Response`].
    #[cfg(feature = "blocking")]
    pub fn execute<E: Endpoint>(&self, endpoint: &E) -> Result<E::Response> {
        self.send(endpoint.request()?)
    }

    #[cfg(feature = "async")]
    fn build(&self, request: Request) -> Result<reqwest::RequestBuilder> {
        let url = request.url(self.config.site_url(request.site))?;
        let builder = self
            .inner
            .request(request.method, url)
            .headers(request.headers);
        Ok(match request.body {
            Some(body) => builder.body(body),
            None => builder,
        })
    }

    #[cfg(feature = "blocking")]
    fn build(&self, request: Request) -> Result<reqwest::blocking::RequestBuilder> {
        let url = request.url(self.config.site_url(request.site))?;
        let builder = self
            .inner
            .request(request.method, url)
            .headers(request.headers);
        Ok(match request.body {
            Some(body) => builder.body(body),
            None => builder,
        })
    }
}

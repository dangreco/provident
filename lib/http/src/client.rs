// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::config::ClientConfig;
use crate::error::Result;
use crate::request::{Endpoint, Request, decode};

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

    #[cfg(feature = "async")]
    pub async fn execute<E: Endpoint>(&self, endpoint: &E) -> Result<E::Response> {
        self.send(endpoint.request()?).await
    }

    #[cfg(feature = "blocking")]
    pub fn execute<E: Endpoint>(&self, endpoint: &E) -> Result<E::Response> {
        self.send(endpoint.request()?)
    }

    #[cfg(feature = "async")]
    fn build(&self, request: Request) -> Result<reqwest::RequestBuilder> {
        let url = request.url(&self.config.base_url)?;
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
        let url = request.url(&self.config.base_url)?;
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

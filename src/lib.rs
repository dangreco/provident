// SPDX-License-Identifier: GPL-3.0-only

//! High-level Provident API client: `client.auth.login(..)`, `client.account.meters()`,
//! `client.usage.series(..)`, ….
//!
//! Async by default; with `default-features = false, features = ["blocking"]` every endpoint
//! method is a plain blocking `fn` instead (via `maybe_async`). Exactly one must be enabled.
//!
//! # Layout
//!
//! Endpoints are grouped by feature area, not by URL: each area is a public field of
//! [`Client`] (e.g. [`Client::auth`]) backed by `src/<area>/mod.rs`, with one file per
//! operation (`src/auth/login.rs`) holding an `impl` block for the area. Request/response
//! types come from [`crate::models`], whose tree mirrors the URL; the area module is where
//! the friendly name (`login`) is mapped to the wire endpoint (`LoginService.aspx/ProcessLogin`).
//!
//! All areas share one HTTP client, so cookies set by [`auth::Auth::login`] authorize every
//! later call.

#[cfg(all(feature = "async", feature = "blocking"))]
compile_error!(
    "features `async` and `blocking` are mutually exclusive; use `default-features = false` with `blocking`"
);
#[cfg(not(any(feature = "async", feature = "blocking")))]
compile_error!("enable exactly one of the `async` or `blocking` features");

pub mod account;
pub mod auth;
mod error;
pub mod http;
pub mod models;
pub mod usage;

pub use error::{Error, Result};
pub use http::ClientConfig;

/// Entry point. Cheap to clone; clones share connections and the session.
#[derive(Debug, Clone)]
pub struct Client {
    /// Session check and sign-in (`/login/LoginService.aspx`).
    pub auth: auth::Auth,
    /// Data for the signed-in account (`/secure/…`); requires [`auth::Auth::login`].
    pub account: account::Account,
    /// Consumption history for meters and groups; requires [`auth::Auth::login`].
    pub usage: usage::Usage,
}

impl Client {
    /// Client for the default base URL ([`http::constants::DEFAULT_BASE_URL`]).
    pub fn new() -> Result<Self> {
        Self::with_config(ClientConfig::default())
    }

    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let http = http::Client::new(config)?;
        Ok(Self {
            auth: auth::Auth::new(http.clone()),
            account: account::Account::new(http.clone()),
            usage: usage::Usage::new(http),
        })
    }
}

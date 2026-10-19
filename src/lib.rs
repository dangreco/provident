// SPDX-License-Identifier: GPL-3.0-only

// Layout (for contributors): endpoints are grouped by feature area, not by URL. Each area is a
// public field of `Client` backed by `src/<area>/mod.rs`, with one file per operation (e.g.
// `src/auth/login.rs`) holding an `impl` block for the area. Request/response types live in
// `models`, whose tree mirrors the portal's URLs; the area module maps the friendly name
// (`login`) to the wire endpoint (`LoginService.aspx/ProcessLogin`). All areas share one HTTP
// client, so the session cookie set by `auth.login` authorizes every later call.

// The README is the crate documentation (docs.rs and crates.io show the same text) and its
// example is compiled as a doctest. It is async-only, so blocking builds get a short summary.
#![cfg_attr(feature = "async", doc = include_str!("../README.md"))]
#![cfg_attr(
    not(feature = "async"),
    doc = "Unofficial client for the Provident (MeterConnex) utility metering portal (blocking API)."
)]
#![warn(missing_docs)]

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

    /// Client for a custom [`ClientConfig`] (base URL, timeouts, headers, user agent).
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let http = http::Client::new(config)?;
        Ok(Self {
            auth: auth::Auth::new(http.clone()),
            account: account::Account::new(http.clone()),
            usage: usage::Usage::new(http),
        })
    }
}

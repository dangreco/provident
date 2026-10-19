// SPDX-License-Identifier: GPL-3.0-only

//! Crate-wide constants. Defaults here are applied by [`crate::http::ClientConfig::builder`].

use std::time::Duration;

/// Base URL used unless overridden with `ClientConfig::builder().base_url(..)`.
/// Site root: endpoints live under both `login/` and `secure/`, so paths are root-relative
/// (e.g. `login/LoginService.aspx/ProcessLogin`).
pub const DEFAULT_BASE_URL: &str = "https://provident.meterconnex.com/";

/// `User-Agent` sent with every request unless overridden.
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Total per-request timeout (connect + send + receive).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Timeout for establishing a TCP/TLS connection.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// MIME type used for `Accept` and `Content-Type`.
pub const APPLICATION_JSON: &str = "application/json";

/// Body sent by non-GET [`crate::http::Request::page_method`]s when no arguments are given.
pub const EMPTY_JSON_OBJECT: &[u8] = b"{}";

/// Maximum number of response body bytes kept in [`crate::http::Error`] for diagnostics.
pub const ERROR_BODY_PREVIEW_LIMIT: usize = 4096;

// SPDX-License-Identifier: GPL-3.0-only

//! JSON HTTP client foundation.
//!
//! [`Client`] is async with feature `async` (default) or blocking with feature `blocking`
//! (`default-features = false, features = ["blocking"]`). Exactly one must be enabled.
//!
//! API code describes operations as [`Endpoint`]s (or ad-hoc [`Request`]s) and runs them
//! through [`Client::execute`] / [`Client::send`].

#[cfg(all(feature = "async", feature = "blocking"))]
compile_error!(
    "features `async` and `blocking` are mutually exclusive; use `default-features = false` with `blocking`"
);
#[cfg(not(any(feature = "async", feature = "blocking")))]
compile_error!("enable exactly one of the `async` or `blocking` features");

mod client;
mod config;
pub mod constants;
mod error;
mod request;

pub use client::Client;
pub use config::{ClientConfig, ClientConfigBuilder};
pub use error::{Error, Result};
pub use request::{Endpoint, Request, RequestBuilder};
pub use reqwest::header::{self, HeaderName, HeaderValue};
pub use reqwest::{Method, StatusCode};

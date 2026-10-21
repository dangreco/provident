// SPDX-License-Identifier: GPL-3.0-only

//! JSON-over-HTTP transport shared by every endpoint: client configuration, request building,
//! the `{"d": ...}` page-method envelope, and HTTP-level errors.
//!
//! [`crate::Client`] wraps this; reach for it directly only to change the base URL, timeouts
//! or headers ([`ClientConfig`]) or to inspect transport errors ([`Error`]).

mod client;
mod config;
pub mod constants;
mod error;
mod request;

pub use client::Client;
pub use config::{ClientConfig, ClientConfigBuilder};
pub use error::{Error, Result};
pub use request::{Endpoint, Request, RequestBuilder, Site};
pub use reqwest::header::{self, HeaderName, HeaderValue};
pub use reqwest::{Method, StatusCode};

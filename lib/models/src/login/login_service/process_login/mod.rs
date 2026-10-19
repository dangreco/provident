// SPDX-License-Identifier: GPL-3.0-only

//! `POST /login/LoginService.aspx/ProcessLogin`: sign in with username and password.
//!
//! Always answers 200; failure is reported in [`ProcessLoginResponse`]. On success the server
//! sets the `.ASPXAUTH` cookie.

mod req;
mod res;

pub use req::{ProcessLoginRequest, ProcessLoginRequestBuilder};
pub use res::{ProcessLoginResponse, ProcessLoginResult};

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "login/LoginService.aspx/ProcessLogin";

/// HTTP verb the `[WebMethod]` accepts.
pub const METHOD: http::Method = http::Method::POST;

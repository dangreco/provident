// SPDX-License-Identifier: GPL-3.0-only

//! `GET /login/LoginService.aspx/CheckLogin`: whether the client's cookies hold a valid session.

mod req;
mod res;

pub use req::CheckLoginRequest;
pub use res::CheckLoginResponse;

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "login/LoginService.aspx/CheckLogin";

/// HTTP verb the `[WebMethod]` accepts (`UseHttpGet = true`; POST answers 500).
pub const METHOD: http::Method = http::Method::GET;

// SPDX-License-Identifier: GPL-3.0-only

//! `POST /secure/AccountInfo.aspx/GetKey`: the account's API key, released on re-entering the
//! account password. Always answers 200; a wrong password is reported in the body.

mod req;
mod res;

pub use req::GetKeyRequest;
pub use res::{GetKeyResponse, GetKeyResult};

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "secure/AccountInfo.aspx/GetKey";

/// HTTP method the endpoint accepts.
pub const METHOD: reqwest::Method = reqwest::Method::POST;

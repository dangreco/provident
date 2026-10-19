// SPDX-License-Identifier: GPL-3.0-only

//! `GET /secure/AccountInfo.aspx/GetDetails`: the signed-in user's profile.

mod req;
mod res;

pub use req::GetDetailsRequest;
pub use res::{AccountDetails, GetDetailsResponse};

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "secure/AccountInfo.aspx/GetDetails";

/// HTTP verb the `[WebMethod]` accepts (`UseHttpGet = true`; POST answers 500).
pub const METHOD: http::Method = http::Method::GET;

// SPDX-License-Identifier: GPL-3.0-only

//! `POST /secure/Dashboard/Default.aspx/GetUtilities`: utilities metered for the signed-in
//! account.

mod req;
mod res;

pub use req::GetUtilitiesRequest;
pub use res::GetUtilitiesResponse;

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "secure/Dashboard/Default.aspx/GetUtilities";

/// HTTP verb the `[WebMethod]` accepts (GET answers 500).
pub const METHOD: http::Method = http::Method::POST;

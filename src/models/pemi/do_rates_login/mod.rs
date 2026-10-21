// SPDX-License-Identifier: GPL-3.0-only

//! `POST /do-rates-login`: start a rates-page session on Provident's corporate site.
//!
//! Form-encoded `LocationID` + `CustomerNumber` (yes: the *metering* portal's username and
//! password, respectively — the two sites share credentials with the fields swapped). Any
//! non-empty `CustomerNumber` is accepted; only `LocationID` is validated. The response sets
//! the `PHPSESSID` cookie that authorizes `rates-and-fees`.

mod res;

pub use res::RatesLoginResponse;

/// Root-relative path, joined to the rates site base URL.
pub const PATH: &str = "do-rates-login";

/// HTTP verb the endpoint accepts.
pub const METHOD: reqwest::Method = reqwest::Method::POST;

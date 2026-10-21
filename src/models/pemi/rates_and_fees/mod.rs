// SPDX-License-Identifier: GPL-3.0-only

//! `GET /rates-and-fees`: the account location's published electricity rates and Provident
//! fees, as HTML tables. Requires the `PHPSESSID` cookie set by `do-rates-login`; without it
//! the page answers the login form instead. Types live in [`parsed`]; this module holds the
//! wire path.

mod parse;

pub mod parsed;

/// Root-relative path, joined to the rates site base URL.
pub const PATH: &str = "rates-and-fees";

/// HTTP verb the page accepts.
pub const METHOD: reqwest::Method = reqwest::Method::GET;

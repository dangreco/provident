// SPDX-License-Identifier: GPL-3.0-only

//! `GET /api/internal/graphs/quickgraphs`: consumption series for meters and groups over a
//! date range, one series per utility.
//!
//! Resolution is chosen by the server from the range length: up to 7 days → hourly (24 values
//! per day), longer → daily (one per day). Days without data are `0.0`, not missing.

mod req;
mod res;

pub use req::QuickGraphsRequest;
pub use res::{QuickGraphsResponse, Series};

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "api/internal/graphs/quickgraphs";

pub const METHOD: http::Method = http::Method::GET;

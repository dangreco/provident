// SPDX-License-Identifier: GPL-3.0-only

//! `GET /api/internal/graphs/quickgraphs`: consumption series for meters and groups over a
//! date range, one series per utility.
//!
//! # Resolution
//!
//! Chosen by the server from the range length: up to 7 days → hourly (24 values per day),
//! longer → daily (one per day). Daily values equal the sum of that day's hourly values.
//!
//! # Time
//!
//! Dates are calendar days with no time zone attached, and every day has exactly 24 hourly
//! values, including daylight-saving transition days. The portal's own charts plot value `i`
//! at `start + i` hours with UTC display. What wall-clock hour a bucket covers is not stated by
//! the server and differs by meter (observed on one suite in Toronto, so treat as indicative):
//!
//! - Electricity buckets behave like local wall-clock hours: the overnight trough stays at
//!   `05` across EST/EDT, and the daily routine does not shift at DST changes.
//! - Water meters (cold, hot) behave like a fixed offset: their routine stays put at the
//!   changeover too, but against electricity they run 2 buckets earlier in winter (EST) and
//!   1–2 earlier in summer (EDT). Water bucket `i` is plausibly UTC−7 rather than local time.
//!
//! So align water and electricity by real time only with care: on this site, water bucket
//! `i + 2` (winter) and roughly `i + 1` (summer) lines up with electricity bucket `i`.
//!
//! # Missing data
//!
//! Hourly series report `0.0` where readings are missing. Daily series fill gaps by spreading
//! the consumption across the missing days (several identical values in a row), so a day's
//! daily value can be non-zero while its hourly values are all zero.

mod req;
mod res;

pub use req::QuickGraphsRequest;
pub use res::{QuickGraphsResponse, Series};

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "api/internal/graphs/quickgraphs";

pub const METHOD: reqwest::Method = reqwest::Method::GET;

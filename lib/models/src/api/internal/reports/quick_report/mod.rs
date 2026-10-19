// SPDX-License-Identifier: GPL-3.0-only

//! `GET /api/internal/reports/quickreport`: start/end meter readings and consumption totals per
//! meter over a date range, grouped by utility.

mod req;
mod res;

pub use req::QuickReportRequest;
pub use res::{QuickReportResponse, ReportRow, RowSource};

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "api/internal/reports/quickreport";

pub const METHOD: http::Method = http::Method::GET;

// SPDX-License-Identifier: GPL-3.0-only

//! Serde helpers for ASP.NET (`JavaScriptSerializer`) wire formats used by page methods.

pub mod date;
mod json_string;

pub use json_string::JsonString;

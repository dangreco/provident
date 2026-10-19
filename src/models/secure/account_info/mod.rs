// SPDX-License-Identifier: GPL-3.0-only

//! `/secure/AccountInfo.aspx`: the signed-in user's profile and API key.
//!
//! Also hosts `NewKey` (rotates the API key) and `ChangePassword`; not modeled.

pub mod get_details;
pub mod get_key;

/// Root-relative path of the page.
pub const PAGE: &str = "secure/AccountInfo.aspx";

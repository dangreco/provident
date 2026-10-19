// SPDX-License-Identifier: GPL-3.0-only

//! `/login/LoginService.aspx`: session check and sign-in.
//!
//! Signing in sets the `.ASPXAUTH` forms-auth cookie (alongside `ASP.NET_SessionId`), which
//! authorizes later `/secure/` calls from the same client.

pub mod check_login;
pub mod process_login;

/// Root-relative path of the page.
pub const PAGE: &str = "login/LoginService.aspx";

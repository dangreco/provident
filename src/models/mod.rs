// SPDX-License-Identifier: GPL-3.0-only

//! Typed data returned by the Provident (meterconnex) API.
//!
//! Two kinds of endpoint: ASP.NET page methods (`Page.aspx/Method`), whose responses are the
//! `d` value (`crate::http` strips the `{"d": ...}` envelope), and plain JSON REST endpoints
//! under [`api`]. ASP.NET-specific wire formats live in [`aspnet`].
//!
//! # Layout
//!
//! The module tree mirrors the URL, one level per segment, in `snake_case`:
//!
//! ```text
//! https://provident.meterconnex.com/login/LoginService.aspx/ProcessLogin
//!                                   └─┬─┘ └──────┬───────┘ └────┬─────┘
//! src/                                login/  login_service/   process_login/
//!                                     mod.rs  mod.rs (PAGE)    mod.rs (PATH, METHOD)
//!                                                              req.rs  (…Request)
//!                                                              res.rs  (…Response)
//! ```
//!
//! - Directory module (`login`) and page module (`login_service`): `mod.rs` only, no types.
//!   The page's `mod.rs` holds `PAGE`, its root-relative path.
//! - Method module (`process_login`): `PATH` (root-relative, joined to the client base URL)
//!   and `METHOD` (HTTP verb the `[WebMethod]` accepts). `req.rs` holds `<Method>Request`
//!   (JSON body; property names match the C# parameters); `res.rs` holds `<Method>Response`
//!   (the `d` value). Both are re-exported from the method module. Methods without parameters
//!   still get a unit `<Method>Request` so every endpoint has the same shape.
//! - Types shared by several methods go in the nearest common ancestor's own file
//!   (e.g. `login/login_service/common.rs`), never in another method's `req.rs`/`res.rs`.
//!   Domain types used across pages (e.g. [`common::Utility`]) live in [`common`].
//! - REST endpoints follow the same layout with the path's segments as modules (e.g.
//!   `api/internal/metertree/rootnodes` → `api::internal::meter_tree::root_nodes`); `req.rs`
//!   holds query parameters and `res.rs` the bare response body.

pub mod api;
pub mod aspnet;
pub mod common;
pub mod login;
pub mod pemi;
pub mod secure;

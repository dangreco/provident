// SPDX-License-Identifier: GPL-3.0-only

//! Typed data returned by the Provident (meterconnex) API.
//!
//! Responses are the page method's `d` value (`provident_http` strips the `{"d": ...}`
//! envelope). ASP.NET-specific wire formats live in [`aspnet`].
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

pub mod aspnet;
pub mod login;

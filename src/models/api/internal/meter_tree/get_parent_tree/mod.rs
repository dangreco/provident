// SPDX-License-Identifier: GPL-3.0-only

//! `GET /api/internal/metertree/getparenttree?nodeId=<NodeId>`: the groups above a node, from
//! the tree root down to the user's own group (e.g. `MeterConnex Root` … site … floor … unit).
//!
//! Unlike the other tree endpoints, the returned nodes' `parent` is not their real parent: every
//! node reports the user's top-level group. Order is root-first.

mod req;
mod res;

pub use req::GetParentTreeRequest;
pub use res::GetParentTreeResponse;

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "api/internal/metertree/getparenttree";

/// HTTP method the endpoint accepts.
pub const METHOD: reqwest::Method = reqwest::Method::GET;

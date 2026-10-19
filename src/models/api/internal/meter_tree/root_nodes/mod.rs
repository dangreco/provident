// SPDX-License-Identifier: GPL-3.0-only

//! `GET /api/internal/metertree/rootnodes?depth=<n>`: the user's top-level groups, and their
//! descendants down to `depth` levels.

mod req;
mod res;

pub use req::RootNodesRequest;
pub use res::RootNodesResponse;

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "api/internal/metertree/rootnodes";

pub const METHOD: reqwest::Method = reqwest::Method::GET;

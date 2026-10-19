// SPDX-License-Identifier: GPL-3.0-only

//! `GET /api/internal/metertree/getchildren?groupId=<n>`: direct children of a group, used to
//! expand nodes with [`super::TreeNode::has_unloaded_children`].

mod req;
mod res;

pub use req::GetChildrenRequest;
pub use res::GetChildrenResponse;

/// Root-relative path, joined to the client base URL.
pub const PATH: &str = "api/internal/metertree/getchildren";

/// HTTP method the endpoint accepts.
pub const METHOD: reqwest::Method = reqwest::Method::GET;

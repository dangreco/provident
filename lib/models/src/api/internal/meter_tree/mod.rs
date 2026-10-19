// SPDX-License-Identifier: GPL-3.0-only

//! `/api/internal/metertree/`: the signed-in user's groups and metering points, as shown in the
//! portal's meter picker. Each response is a flat list of [`TreeNode`]s.
//!
//! Nodes are identified by [`crate::common::NodeId`] (`GROUP:<n>`, `MP:<n>`), the same ids
//! other endpoints accept. A group whose children were not included has
//! [`TreeNode::has_unloaded_children`] set; load them with [`get_children`]. [`get_parent_tree`]
//! lists a node's ancestors (organization, site, floor, …), which the user can see but not
//! expand.

mod common;
pub mod get_children;
pub mod get_parent_tree;
pub mod root_nodes;

pub use common::{InvalidMeter, Meter, NodeAnchor, TreeNode};

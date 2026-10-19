// SPDX-License-Identifier: GPL-3.0-only

//! `/api/internal/metertree/`: the signed-in user's groups and metering points, as shown in the
//! portal's meter picker. Each response is a flat list of [`TreeNode`]s linked by `parent`.
//!
//! A group whose children were not included has [`TreeNode::has_unloaded_children`] set; load
//! them with [`get_children`].

mod common;
pub mod get_children;
pub mod root_nodes;

pub use common::{InvalidMeter, Meter, NodeAnchor, TreeNode};

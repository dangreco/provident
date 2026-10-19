// SPDX-License-Identifier: GPL-3.0-only

//! Domain types shared by endpoints across pages.

mod node_id;
mod utility;

pub use node_id::{GroupId, InvalidNodeId, MeterId, NodeId};
pub use utility::{UnknownUtility, Utility};

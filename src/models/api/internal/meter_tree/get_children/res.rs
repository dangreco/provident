// SPDX-License-Identifier: GPL-3.0-only

use super::super::TreeNode;

/// The group's direct children; any of them may in turn have unloaded children.
pub type GetChildrenResponse = Vec<TreeNode>;

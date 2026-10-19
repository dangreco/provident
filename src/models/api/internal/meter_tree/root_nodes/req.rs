// SPDX-License-Identifier: GPL-3.0-only

use serde::Serialize;

/// Query string parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RootNodesRequest {
    /// Levels to include: `1` = top-level groups only, `2` = groups and their children.
    pub depth: u32,
}

/// `depth = 2`, as the portal's meter picker requests.
impl Default for RootNodesRequest {
    fn default() -> Self {
        Self { depth: 2 }
    }
}

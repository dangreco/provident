// SPDX-License-Identifier: GPL-3.0-only

use serde::Serialize;

use crate::common::NodeId;

/// Query string parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GetParentTreeRequest {
    /// Prefixed node id (`MP:<n>` or `GROUP:<n>`).
    #[serde(rename = "nodeId")]
    pub node_id: NodeId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::MeterId;

    #[test]
    fn encodes_prefixed_node_id() {
        let query = serde_urlencoded::to_string(GetParentTreeRequest {
            node_id: NodeId::Meter(MeterId(42)),
        })
        .unwrap();
        assert_eq!(query, "nodeId=MP%3A42");
    }
}

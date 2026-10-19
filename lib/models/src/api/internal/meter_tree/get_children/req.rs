// SPDX-License-Identifier: GPL-3.0-only

use serde::Serialize;

/// Query string parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GetChildrenRequest {
    /// Numeric group id ([`super::super::TreeNode::group_id`]); the server rejects the
    /// prefixed `GROUP:<n>` form with 400.
    #[serde(rename = "groupId")]
    pub group_id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_server_parameter_name() {
        let query = serde_urlencoded::to_string(GetChildrenRequest { group_id: 7 }).unwrap();
        assert_eq!(query, "groupId=7");
    }
}

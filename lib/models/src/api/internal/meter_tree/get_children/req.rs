// SPDX-License-Identifier: GPL-3.0-only

use serde::Serialize;

use crate::common::GroupId;

/// Query string parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GetChildrenRequest {
    /// Bare group number; the server rejects the prefixed `GROUP:<n>` form with 400, and groups
    /// outside the user's own tree with 401.
    #[serde(rename = "groupId")]
    pub group_id: GroupId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_bare_group_number() {
        let query = serde_urlencoded::to_string(GetChildrenRequest {
            group_id: GroupId(7),
        })
        .unwrap();
        assert_eq!(query, "groupId=7");
    }
}

// SPDX-License-Identifier: GPL-3.0-only

use super::super::TreeNode;

/// Ancestor groups, root-first, ending with the user's own group. See the module docs for the
/// `parent` caveat.
pub type GetParentTreeResponse = Vec<TreeNode>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::GroupId;

    #[test]
    fn decodes_observed_shape() {
        // Shape of an observed response (abridged); `parent` is the user's group on every node.
        let nodes: GetParentTreeResponse = serde_json::from_str(
            r#"[
                {"id":"GROUP:1","text":"MeterConnex Root","parent":"GROUP:9","icon":"fas fa-folder","children":true},
                {"id":"GROUP:5","text":"Tower (12)","parent":"GROUP:9","icon":"far fa-building","type":"site","children":true},
                {"id":"GROUP:9","text":"101","parent":"GROUP:9","icon":"fas fa-folder","children":true}
            ]"#,
        )
        .unwrap();
        let sites: Vec<_> = nodes
            .iter()
            .filter(|n| n.is_site())
            .map(|n| n.text.as_str())
            .collect();
        assert_eq!(sites, ["Tower (12)"]);
        assert_eq!(nodes.last().and_then(TreeNode::group_id), Some(GroupId(9)));
    }
}

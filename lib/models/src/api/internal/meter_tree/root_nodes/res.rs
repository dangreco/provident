// SPDX-License-Identifier: GPL-3.0-only

use super::super::TreeNode;

/// Flat list of nodes, parents before children.
pub type RootNodesResponse = Vec<TreeNode>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::internal::meter_tree::Meter;
    use crate::common::Utility;

    // Same shape as an observed response; identifiers are made up.
    const PAYLOAD: &str = r##"[
        {"id":"GROUP:7","text":"101","parent":"#","icon":"fas fa-folder"},
        {"id":"MP:11","text":"101","parent":"GROUP:7","icon":"/icons/utilities/16/Cold Water.png","a_attr":{"title":"101 [Cold Water - m3] - 1111-1"}},
        {"id":"MP:12","text":"101","parent":"GROUP:7","icon":"/icons/utilities/16/Electricity.png","a_attr":{"title":"101 [Electricity - kWh] - 2222-2"}}
    ]"##;

    #[test]
    fn decodes_observed_shape() {
        let nodes: RootNodesResponse = serde_json::from_str(PAYLOAD).unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].group_id(), Some(7));
        assert!(nodes[0].anchor.is_none() && !nodes[0].has_unloaded_children());

        let meters: Vec<_> = nodes
            .iter()
            .filter(|node| node.meter_id().is_some())
            .map(|node| Meter::try_from(node).unwrap())
            .collect();
        assert_eq!(
            meters.iter().map(|m| m.utility).collect::<Vec<_>>(),
            [Utility::ColdWater, Utility::Electricity]
        );
        assert_eq!(meters[1].unit, "kWh");
    }

    #[test]
    fn decodes_lazy_children_marker() {
        let nodes: RootNodesResponse = serde_json::from_str(
            r##"[{"id":"GROUP:1","text":"Root","parent":"#","icon":"fas fa-folder","children":true}]"##,
        )
        .unwrap();
        assert!(nodes[0].has_unloaded_children());
    }
}

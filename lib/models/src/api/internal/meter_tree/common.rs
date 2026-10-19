// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

use crate::common::{GroupId, MeterId, NodeId, UnknownUtility, Utility};

/// One node of the meter tree (a jsTree node as sent by the server).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: NodeId,
    /// Parent node; `None` for a top-level node (sent as `#`).
    #[serde(with = "parent")]
    pub parent: Option<NodeId>,
    /// Display label: the group/site name, or for meters the unit/suite name.
    pub text: String,
    /// Icon class or path; for meters `/icons/utilities/16/<utility>.png`.
    pub icon: String,
    /// jsTree node type; `site` for buildings, otherwise absent.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Anchor attributes; for meters carries the descriptive [`NodeAnchor::title`].
    #[serde(rename = "a_attr", default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<NodeAnchor>,
    /// `true` when this node has children that were not included in the response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAnchor {
    /// For meters: `<name> [<utility> - <unit>] - <serial>`.
    pub title: String,
}

/// jsTree's root marker for `parent`.
const ROOT: &str = "#";

/// `#` ↔ `None`, otherwise a [`NodeId`].
mod parent {
    use std::borrow::Cow;

    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serializer};

    use super::ROOT;
    use crate::common::NodeId;

    pub fn serialize<S: Serializer>(
        value: &Option<NodeId>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(id) => serializer.collect_str(id),
            None => serializer.serialize_str(ROOT),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<NodeId>, D::Error> {
        let raw = Cow::<'de, str>::deserialize(deserializer)?;
        if raw == ROOT {
            return Ok(None);
        }
        raw.parse().map(Some).map_err(D::Error::custom)
    }
}

impl TreeNode {
    pub fn group_id(&self) -> Option<GroupId> {
        self.id.group()
    }

    pub fn meter_id(&self) -> Option<MeterId> {
        self.id.meter()
    }

    /// Whether this group is a site (building), e.g. `Nobu (587)`.
    pub fn is_site(&self) -> bool {
        self.kind.as_deref() == Some("site")
    }

    /// Whether this node has children that must be loaded separately.
    pub fn has_unloaded_children(&self) -> bool {
        self.children == Some(true)
    }
}

/// A metering point, parsed from a [`TreeNode`] with an `MP:` id.
///
/// [`id`](Self::id) is the key other endpoints take (as [`NodeId::Meter`] in `meterlist=`);
/// [`serial`](Self::serial) is the physical meter number the portal shows as "Meter #".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meter {
    pub id: MeterId,
    /// Group the meter belongs to (typically the unit/suite).
    pub group: Option<GroupId>,
    /// Unit/suite the meter belongs to, e.g. `3105`.
    pub name: String,
    pub utility: Utility,
    /// Unit of measure, e.g. `m3` or `kWh`.
    pub unit: String,
    /// Meter serial number, e.g. `23166694-2`.
    pub serial: String,
}

impl Meter {
    /// This meter as a node id, for endpoints that take `MP:<n>`.
    pub fn node_id(&self) -> NodeId {
        self.id.into()
    }
}

/// A tree node that could not be read as a [`Meter`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidMeter {
    #[error("node {0} is not a metering point")]
    NotAMeter(NodeId),
    #[error("meter {0} has no title")]
    MissingTitle(MeterId),
    #[error("meter {id} has an unrecognized title {title:?}")]
    MalformedTitle { id: MeterId, title: String },
    #[error("meter {id}: {source}")]
    UnknownUtility {
        id: MeterId,
        #[source]
        source: UnknownUtility,
    },
}

impl TryFrom<&TreeNode> for Meter {
    type Error = InvalidMeter;

    fn try_from(node: &TreeNode) -> Result<Self, Self::Error> {
        let id = node.meter_id().ok_or(InvalidMeter::NotAMeter(node.id))?;
        let title = &node
            .anchor
            .as_ref()
            .ok_or(InvalidMeter::MissingTitle(id))?
            .title;
        let (name, utility, unit, serial) =
            parse_title(title).ok_or_else(|| InvalidMeter::MalformedTitle {
                id,
                title: title.clone(),
            })?;
        let utility = utility
            .parse()
            .map_err(|source| InvalidMeter::UnknownUtility { id, source })?;
        Ok(Self {
            id,
            group: node.parent.and_then(NodeId::group),
            name: name.to_owned(),
            utility,
            unit: unit.to_owned(),
            serial: serial.to_owned(),
        })
    }
}

/// Splits `<name> [<utility> - <unit>] - <serial>`.
fn parse_title(title: &str) -> Option<(&str, &str, &str, &str)> {
    let (name, rest) = title.split_once(" [")?;
    let (bracket, serial) = rest.split_once("] - ")?;
    let (utility, unit) = bracket.rsplit_once(" - ")?;
    [name, utility, unit, serial]
        .iter()
        .all(|part| !part.is_empty())
        .then_some((name, utility, unit, serial))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meter_node(title: Option<&str>) -> TreeNode {
        TreeNode {
            id: NodeId::Meter(MeterId(42)),
            parent: Some(NodeId::Group(GroupId(7))),
            text: "101".into(),
            icon: "/icons/utilities/16/Cold Water.png".into(),
            kind: None,
            anchor: title.map(|title| NodeAnchor {
                title: title.into(),
            }),
            children: None,
        }
    }

    #[test]
    fn parses_meter_from_title() {
        let meter = Meter::try_from(&meter_node(Some("101 [Cold Water - m3] - 1234-5"))).unwrap();
        assert_eq!(
            meter,
            Meter {
                id: MeterId(42),
                group: Some(GroupId(7)),
                name: "101".into(),
                utility: Utility::ColdWater,
                unit: "m3".into(),
                serial: "1234-5".into(),
            }
        );
        assert_eq!(meter.node_id().to_string(), "MP:42");
    }

    #[test]
    fn rejects_non_meter_and_bad_titles() {
        let group = TreeNode {
            id: NodeId::Group(GroupId(7)),
            ..meter_node(None)
        };
        assert_eq!(
            Meter::try_from(&group),
            Err(InvalidMeter::NotAMeter(NodeId::Group(GroupId(7))))
        );
        assert_eq!(
            Meter::try_from(&meter_node(None)),
            Err(InvalidMeter::MissingTitle(MeterId(42)))
        );
        for title in [
            "101",
            "101 [Cold Water] - 1",
            "101 [Cold Water - m3]",
            " [Cold Water - m3] - 1",
        ] {
            assert!(
                matches!(
                    Meter::try_from(&meter_node(Some(title))),
                    Err(InvalidMeter::MalformedTitle { .. })
                ),
                "{title}"
            );
        }
        assert!(matches!(
            Meter::try_from(&meter_node(Some("101 [Steam - kg] - 1"))),
            Err(InvalidMeter::UnknownUtility { .. })
        ));
    }

    #[test]
    fn parent_root_marker_and_site_kind() {
        let node: TreeNode = serde_json::from_str(
            r##"{"id":"GROUP:5","text":"Nobu (587)","parent":"#","icon":"far fa-building","type":"site"}"##,
        )
        .unwrap();
        assert_eq!((node.parent, node.group_id()), (None, Some(GroupId(5))));
        assert!(node.is_site() && !node.has_unloaded_children());
        let json = serde_json::to_value(&node).unwrap();
        assert_eq!(
            (json["parent"].as_str(), json["id"].as_str()),
            (Some("#"), Some("GROUP:5"))
        );
    }

    #[test]
    fn rejects_unknown_node_ids() {
        let err = serde_json::from_str::<TreeNode>(
            r##"{"id":"SITE:5","text":"x","parent":"#","icon":"x"}"##,
        );
        assert!(err.is_err());
    }
}

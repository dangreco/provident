// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

use crate::common::{UnknownUtility, Utility};

/// Node id prefix of groups (`GROUP:121497`).
const GROUP_PREFIX: &str = "GROUP:";
/// Node id prefix of metering points (`MP:692294`).
const METER_PREFIX: &str = "MP:";

/// One node of the meter tree (a jsTree node as sent by the server).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode {
    /// `<KIND>:<number>`, e.g. `GROUP:121497` or `MP:692294`.
    pub id: String,
    /// Id of the parent node, or `#` for a top-level node.
    pub parent: String,
    /// Display label (for meters, the unit/suite name).
    pub text: String,
    /// Icon class or path; for meters `/icons/utilities/16/<utility>.png`.
    pub icon: String,
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

impl TreeNode {
    /// Numeric id when this node is a group (`GROUP:<n>`); the `groupId` for
    /// [`super::get_children`].
    pub fn group_id(&self) -> Option<u64> {
        prefixed_id(&self.id, GROUP_PREFIX)
    }

    /// Numeric id when this node is a metering point (`MP:<n>`).
    pub fn meter_id(&self) -> Option<u64> {
        prefixed_id(&self.id, METER_PREFIX)
    }

    /// Whether this node has children that must be loaded separately.
    pub fn has_unloaded_children(&self) -> bool {
        self.children == Some(true)
    }
}

fn prefixed_id(id: &str, prefix: &str) -> Option<u64> {
    id.strip_prefix(prefix)?.parse().ok()
}

/// A metering point, parsed from a [`TreeNode`] with an `MP:` id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meter {
    /// Metering point id (`MP:<id>`).
    pub id: u64,
    /// Unit/suite the meter belongs to, e.g. `3105`.
    pub name: String,
    pub utility: Utility,
    /// Unit of measure, e.g. `m3` or `kWh`.
    pub unit: String,
    /// Meter serial number, e.g. `23166694-2`.
    pub serial: String,
}

/// A tree node that could not be read as a [`Meter`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidMeter {
    #[error("node {0:?} is not a metering point")]
    NotAMeter(String),
    #[error("meter {id:?} has no title")]
    MissingTitle { id: String },
    #[error("meter {id:?} has an unrecognized title {title:?}")]
    MalformedTitle { id: String, title: String },
    #[error("meter {id:?}: {source}")]
    UnknownUtility {
        id: String,
        #[source]
        source: UnknownUtility,
    },
}

impl TryFrom<&TreeNode> for Meter {
    type Error = InvalidMeter;

    fn try_from(node: &TreeNode) -> Result<Self, Self::Error> {
        let id = node
            .meter_id()
            .ok_or_else(|| InvalidMeter::NotAMeter(node.id.clone()))?;
        let title = &node
            .anchor
            .as_ref()
            .ok_or_else(|| InvalidMeter::MissingTitle {
                id: node.id.clone(),
            })?
            .title;
        let (name, utility, unit, serial) =
            parse_title(title).ok_or_else(|| InvalidMeter::MalformedTitle {
                id: node.id.clone(),
                title: title.clone(),
            })?;
        let utility = utility
            .parse()
            .map_err(|source| InvalidMeter::UnknownUtility {
                id: node.id.clone(),
                source,
            })?;
        Ok(Self {
            id,
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
            id: "MP:42".into(),
            parent: "GROUP:7".into(),
            text: "101".into(),
            icon: "/icons/utilities/16/Cold Water.png".into(),
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
                id: 42,
                name: "101".into(),
                utility: Utility::ColdWater,
                unit: "m3".into(),
                serial: "1234-5".into(),
            }
        );
    }

    #[test]
    fn rejects_non_meter_and_bad_titles() {
        let group = TreeNode {
            id: "GROUP:7".into(),
            ..meter_node(None)
        };
        assert_eq!(
            Meter::try_from(&group),
            Err(InvalidMeter::NotAMeter("GROUP:7".into()))
        );
        assert!(matches!(
            Meter::try_from(&meter_node(None)),
            Err(InvalidMeter::MissingTitle { .. })
        ));
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
    fn node_ids_and_lazy_children() {
        let mut node = meter_node(None);
        assert_eq!((node.meter_id(), node.group_id()), (Some(42), None));
        node.id = "GROUP:7".into();
        assert_eq!((node.meter_id(), node.group_id()), (None, Some(7)));
        node.id = "GROUP:x".into();
        assert_eq!(node.group_id(), None);
        assert!(!node.has_unloaded_children());
        node.children = Some(true);
        assert!(node.has_unloaded_children());
    }
}

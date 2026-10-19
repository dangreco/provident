// SPDX-License-Identifier: GPL-3.0-only

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Metering point id: the number in a `MP:<n>` node id, and the `ID` column of usage reports.
///
/// Serializes as the bare number; endpoints taking node ids want [`NodeId::Meter`] instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MeterId(pub u64);

/// Group id: the number in a `GROUP:<n>` node id, as `getchildren?groupId=` expects it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(pub u64);

impl fmt::Display for MeterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A node of the meter tree, in the prefixed form the portal uses for node ids, `nodeId=` and
/// `meterlist=` (`MP:692294`, `GROUP:121497`).
///
/// Sites (buildings) are groups too; they differ only in the tree node's type.
///
/// ```
/// use provident_models::common::{GroupId, MeterId, NodeId};
///
/// let node: NodeId = "MP:692294".parse().unwrap();
/// assert_eq!(node, NodeId::Meter(MeterId(692294)));
/// assert_eq!(NodeId::from(GroupId(7)).to_string(), "GROUP:7");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeId {
    Group(GroupId),
    Meter(MeterId),
}

const GROUP_PREFIX: &str = "GROUP";
const METER_PREFIX: &str = "MP";

impl NodeId {
    pub const fn group(self) -> Option<GroupId> {
        match self {
            Self::Group(id) => Some(id),
            Self::Meter(_) => None,
        }
    }

    pub const fn meter(self) -> Option<MeterId> {
        match self {
            Self::Meter(id) => Some(id),
            Self::Group(_) => None,
        }
    }
}

impl From<GroupId> for NodeId {
    fn from(id: GroupId) -> Self {
        Self::Group(id)
    }
}

impl From<MeterId> for NodeId {
    fn from(id: MeterId) -> Self {
        Self::Meter(id)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Group(id) => write!(f, "{GROUP_PREFIX}:{id}"),
            Self::Meter(id) => write!(f, "{METER_PREFIX}:{id}"),
        }
    }
}

/// Not a `GROUP:<n>` or `MP:<n>` node id.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid node id {0:?}")]
pub struct InvalidNodeId(pub String);

impl FromStr for NodeId {
    type Err = InvalidNodeId;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || InvalidNodeId(s.to_owned());
        let (prefix, number) = s.split_once(':').ok_or_else(invalid)?;
        // `u64::from_str` accepts a leading `+`; the portal never sends one.
        if !number.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
        let number = number.parse().map_err(|_| invalid())?;
        match prefix {
            GROUP_PREFIX => Ok(Self::Group(GroupId(number))),
            METER_PREFIX => Ok(Self::Meter(MeterId(number))),
            _ => Err(invalid()),
        }
    }
}

impl Serialize for NodeId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Cow::<'de, str>::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_both_kinds() {
        for (text, node) in [
            ("MP:692294", NodeId::Meter(MeterId(692294))),
            ("GROUP:121497", NodeId::Group(GroupId(121497))),
        ] {
            assert_eq!(text.parse::<NodeId>().unwrap(), node);
            assert_eq!(node.to_string(), text);
            let json = serde_json::to_string(&node).unwrap();
            assert_eq!(json, format!("{text:?}"));
            assert_eq!(serde_json::from_str::<NodeId>(&json).unwrap(), node);
        }
    }

    #[test]
    fn rejects_other_forms() {
        // Bare numbers are what `getchildren` takes, but not valid node ids.
        for text in [
            "692294", "MP:", "MP:x", "MP:+1", "MP:-1", "mp:1", "SITE:1", "#", "",
        ] {
            assert_eq!(
                text.parse::<NodeId>(),
                Err(InvalidNodeId(text.into())),
                "{text}"
            );
        }
    }

    #[test]
    fn plain_ids_serialize_as_numbers() {
        assert_eq!(serde_json::to_string(&MeterId(5)).unwrap(), "5");
        assert_eq!(serde_json::to_string(&GroupId(7)).unwrap(), "7");
    }
}

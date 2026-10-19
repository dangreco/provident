// SPDX-License-Identifier: GPL-3.0-only

//! Types shared by the usage endpoints ([`super::graphs`], [`super::reports`]).

use std::fmt;

use serde::{Serialize, Serializer};

use crate::common::NodeId;

/// The `meterlist` query parameter: meters and/or groups, as comma-separated prefixed node ids
/// (`MP:692294,GROUP:121497`). A group expands to every meter below it.
///
/// Must not be empty: the server answers 400. Bare numbers are silently accepted but match
/// nothing, which is why this only holds [`NodeId`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MeterList(pub Vec<NodeId>);

impl MeterList {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Into<NodeId>> FromIterator<T> for MeterList {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<NodeId>> From<T> for MeterList {
    fn from(node: T) -> Self {
        Self(vec![node.into()])
    }
}

impl fmt::Display for MeterList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, node) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{node}")?;
        }
        Ok(())
    }
}

impl Serialize for MeterList {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// `chrono::NaiveDate` as `YYYY-MM-DD`, the format every usage endpoint uses.
pub(crate) mod date {
    use chrono::NaiveDate;
    use serde::Serializer;

    pub fn serialize<S: Serializer>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&date.format("%Y-%m-%d"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{GroupId, MeterId};

    #[test]
    fn joins_prefixed_ids_with_commas() {
        let list = MeterList(vec![NodeId::Meter(MeterId(1)), NodeId::Group(GroupId(2))]);
        assert_eq!(list.to_string(), "MP:1,GROUP:2");
        assert_eq!(MeterList::from(MeterId(5)).to_string(), "MP:5");
        let collected: MeterList = [MeterId(1), MeterId(3)].into_iter().collect();
        assert_eq!(collected.to_string(), "MP:1,MP:3");
    }
}

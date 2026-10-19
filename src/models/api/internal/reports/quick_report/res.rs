// SPDX-License-Identifier: GPL-3.0-only

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::models::common::{GroupId, MeterId};

/// Rows keyed by utility label (`Cold Water (m3)`). Empty when nothing matched.
pub type QuickReportResponse = BTreeMap<String, Vec<ReportRow>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportRow {
    /// Suite/unit name, e.g. `3105`.
    #[serde(rename = "Name")]
    pub name: String,
    /// Meter serial; empty for aggregated rows.
    #[serde(rename = "Meter #")]
    pub serial: String,
    /// Meter reading at the end of the range. `None` when there is no data.
    #[serde(rename = "Current")]
    pub current: Option<f64>,
    /// Meter reading at the start of the range.
    #[serde(rename = "Previous")]
    pub previous: Option<f64>,
    /// Consumption over the range (`current - previous`).
    #[serde(rename = "Total")]
    pub total: Option<f64>,
    #[serde(rename = "Daily Average")]
    pub daily_average: Option<f64>,
    /// Meter id, or the negated group id for aggregated rows; see [`Self::source`].
    #[serde(rename = "ID")]
    pub id: i64,
    /// Readings are missing or incomplete for the range; values may be `None`.
    #[serde(rename = "Error")]
    pub error: bool,
}

/// What a [`ReportRow`] summarizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowSource {
    Meter(MeterId),
    /// Aggregated row (`aggregateGroups=true`).
    Group(GroupId),
}

impl ReportRow {
    pub fn source(&self) -> RowSource {
        if self.id < 0 {
            RowSource::Group(GroupId(self.id.unsigned_abs()))
        } else {
            RowSource::Meter(MeterId(self.id.unsigned_abs()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_meter_and_aggregate_rows() {
        let report: QuickReportResponse = serde_json::from_str(
            r#"{
                "Cold Water (m3)":[{"Name":"101","Meter #":"1111-1","Current":110.777,"Previous":109.227,"Total":1.55,"Daily Average":0.221,"ID":11,"Error":false}],
                "Hot Water (m3)":[{"Name":"101","Meter #":"","Current":93.154,"Previous":89.986,"Total":3.168,"Daily Average":0.106,"ID":-7,"Error":false}]
            }"#,
        )
        .unwrap();
        let cold = &report["Cold Water (m3)"][0];
        assert_eq!(
            (cold.source(), cold.total),
            (RowSource::Meter(MeterId(11)), Some(1.55))
        );
        assert_eq!(
            report["Hot Water (m3)"][0].source(),
            RowSource::Group(GroupId(7))
        );
    }

    #[test]
    fn decodes_row_without_data() {
        let row: ReportRow = serde_json::from_str(
            r#"{"Name":"101","Meter #":"1111-1","Current":null,"Previous":null,"Total":null,"Daily Average":null,"ID":11,"Error":true}"#,
        )
        .unwrap();
        assert!(row.error && row.current.is_none() && row.total.is_none());
    }
}

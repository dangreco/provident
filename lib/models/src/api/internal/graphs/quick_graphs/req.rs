// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDate;
use serde::Serialize;

use crate::api::internal::usage::{MeterList, date};

/// Query string parameters. All are required; omitting any answers 404.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, bon::Builder)]
pub struct QuickGraphsRequest {
    #[serde(rename = "meterlist")]
    #[builder(into)]
    pub meters: MeterList,
    /// First day included.
    #[serde(rename = "startDate", with = "date")]
    pub start: NaiveDate,
    /// Day after the last one included (exclusive). Must be after `start`, else 400.
    #[serde(rename = "endDate", with = "date")]
    pub end: NaiveDate,
    /// Merge each group's meters into one series per utility.
    #[serde(rename = "aggregateGroups")]
    #[builder(default)]
    pub aggregate_groups: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{GroupId, MeterId, NodeId};

    #[test]
    fn encodes_server_parameter_names_and_formats() {
        let req = QuickGraphsRequest::builder()
            .meters(MeterList(vec![
                NodeId::Meter(MeterId(1)),
                NodeId::Group(GroupId(2)),
            ]))
            .start(NaiveDate::from_ymd_opt(2026, 10, 1).unwrap())
            .end(NaiveDate::from_ymd_opt(2026, 10, 8).unwrap())
            .build();
        assert_eq!(
            serde_urlencoded::to_string(&req).unwrap(),
            "meterlist=MP%3A1%2CGROUP%3A2&startDate=2026-10-01&endDate=2026-10-08&aggregateGroups=false"
        );
    }
}

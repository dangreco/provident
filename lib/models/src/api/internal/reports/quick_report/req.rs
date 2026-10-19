// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDate;
use serde::Serialize;

use crate::api::internal::usage::{MeterList, date};

/// Query string parameters. All are required; omitting any answers 404.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, bon::Builder)]
pub struct QuickReportRequest {
    #[serde(rename = "meterlist")]
    #[builder(into)]
    pub meters: MeterList,
    /// First day included.
    #[serde(rename = "startDate", with = "date")]
    pub start: NaiveDate,
    /// Day after the last one included (exclusive). Must be after `start`, else 400.
    #[serde(rename = "endDate", with = "date")]
    pub end: NaiveDate,
    /// One row per group and utility instead of per meter.
    #[serde(rename = "aggregateGroups")]
    #[builder(default)]
    pub aggregate_groups: bool,
    /// Include meters that are no longer active.
    #[serde(rename = "includeDeactivated")]
    #[builder(default)]
    pub include_deactivated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::MeterId;

    #[test]
    fn encodes_server_parameter_names() {
        let req = QuickReportRequest::builder()
            .meters(MeterId(1))
            .start(NaiveDate::from_ymd_opt(2026, 9, 1).unwrap())
            .end(NaiveDate::from_ymd_opt(2026, 10, 1).unwrap())
            .include_deactivated(true)
            .build();
        assert_eq!(
            serde_urlencoded::to_string(&req).unwrap(),
            "meterlist=MP%3A1&startDate=2026-09-01&endDate=2026-10-01&aggregateGroups=false&includeDeactivated=true"
        );
    }
}

// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

/// One series per utility present among the requested meters. Empty when nothing matched.
pub type QuickGraphsResponse = Vec<Series>;

/// Consumption series of one utility, in one unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
    /// Utility with its unit, as a display label: `Cold Water (m3)`.
    pub utility: String,
    /// Unit of measure, e.g. `m3` or `kWh`.
    pub units: String,
    /// `<suite>|<group>|<site>`, e.g. `3105|3105|Nobu (587)`; just `<suite>` when aggregated.
    pub name: String,
    /// Site (building) name.
    pub site: String,
    /// Consumption per interval (hourly or daily, see the module docs), oldest first.
    pub data: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_observed_shape() {
        let series: QuickGraphsResponse = serde_json::from_str(
            r#"[{"utility":"Cold Water (m3)","units":"m3","name":"101|101|Tower (12)","data":[0.005,0.0,0.0],"site":"Tower (12)"}]"#,
        )
        .unwrap();
        assert_eq!(series[0].units, "m3");
        assert_eq!(series[0].data, [0.005, 0.0, 0.0]);
    }
}

// SPDX-License-Identifier: GPL-3.0-only

//! Read-only Python views of the crate's response types. Each wraps the Rust value and exposes
//! its fields as properties; ids that other calls take are exposed as node-id strings.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use chrono::NaiveDate;
use provident::models::api::internal::graphs::quick_graphs;
use provident::models::api::internal::meter_tree;
use provident::models::api::internal::reports::quick_report::{self, RowSource};
use provident::models::common::{NodeId, Utility as RsUtility};
use provident::models::pemi::rates_and_fees::parsed;
use provident::models::secure::account_info::get_details;
use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyType;

/// Canonical portal name of every utility, in declaration order.
pub fn utility_names() -> Vec<&'static str> {
    RsUtility::ALL.iter().map(|u| u.as_str()).collect()
}

/// Converts to the Python `provident.Utility` enum (a `StrEnum` keyed by portal name).
pub struct Utility(pub RsUtility);

impl<'py> IntoPyObject<'py> for Utility {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        static ENUM: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        ENUM.import(py, "provident._types", "Utility")?
            .call1((self.0.as_str(),))
    }
}

/// A metering point on the account.
#[pyclass(frozen, eq, module = "provident")]
#[derive(PartialEq)]
pub struct Meter(pub meter_tree::Meter);

#[pymethods]
impl Meter {
    /// Metering point id (the number in `MP:<id>`).
    #[getter]
    fn id(&self) -> u64 {
        self.0.id.0
    }

    /// This meter as a node id (`MP:<id>`), as `usage` calls take it.
    #[getter]
    fn node_id(&self) -> String {
        self.0.node_id().to_string()
    }

    /// Id of the group (unit/suite) the meter belongs to.
    #[getter]
    fn group(&self) -> Option<u64> {
        self.0.group.map(|g| g.0)
    }

    /// The meter's group as a node id (`GROUP:<id>`).
    #[getter]
    fn group_node_id(&self) -> Option<String> {
        self.0.group.map(|g| NodeId::from(g).to_string())
    }

    /// Unit/suite name, e.g. `3105`.
    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }

    /// Utility this meter measures.
    #[getter]
    fn utility(&self) -> Utility {
        Utility(self.0.utility)
    }

    /// Unit of measure, e.g. `m3` or `kWh`.
    #[getter]
    fn unit(&self) -> &str {
        &self.0.unit
    }

    /// Meter serial number ("Meter #"), e.g. `23166694-2`.
    #[getter]
    fn serial(&self) -> &str {
        &self.0.serial
    }

    fn __repr__(&self) -> String {
        let m = &self.0;
        format!(
            "Meter(id={}, name={:?}, utility={:?}, unit={:?}, serial={:?})",
            m.id,
            m.name,
            m.utility.as_str(),
            m.unit,
            m.serial
        )
    }
}

/// A node of the meter tree: a group, site (building), or metering point.
#[pyclass(frozen, eq, module = "provident")]
#[derive(PartialEq)]
pub struct TreeNode(pub meter_tree::TreeNode);

#[pymethods]
impl TreeNode {
    /// Node id: `MP:<n>` or `GROUP:<n>`.
    #[getter]
    fn id(&self) -> String {
        self.0.id.to_string()
    }

    /// Parent node id; `None` for a top-level node.
    #[getter]
    fn parent(&self) -> Option<String> {
        self.0.parent.map(|p| p.to_string())
    }

    /// Display label: the group/site name, or for meters the unit/suite name.
    #[getter]
    fn text(&self) -> &str {
        &self.0.text
    }

    /// Icon class or path.
    #[getter]
    fn icon(&self) -> &str {
        &self.0.icon
    }

    /// jsTree node type; `site` for buildings, otherwise `None`.
    #[getter]
    fn kind(&self) -> Option<&str> {
        self.0.kind.as_deref()
    }

    /// Tooltip; for meters `<name> [<utility> - <unit>] - <serial>`.
    #[getter]
    fn title(&self) -> Option<&str> {
        self.0.anchor.as_ref().map(|a| a.title.as_str())
    }

    /// The number in `GROUP:<n>`, if this node is a group.
    #[getter]
    fn group_id(&self) -> Option<u64> {
        self.0.group_id().map(|g| g.0)
    }

    /// The number in `MP:<n>`, if this node is a metering point.
    #[getter]
    fn meter_id(&self) -> Option<u64> {
        self.0.meter_id().map(|m| m.0)
    }

    /// Whether this group is a site (building).
    #[getter]
    fn is_site(&self) -> bool {
        self.0.is_site()
    }

    /// Whether this node has children that were not included in the response.
    #[getter]
    fn has_unloaded_children(&self) -> bool {
        self.0.has_unloaded_children()
    }

    fn __repr__(&self) -> String {
        let n = &self.0;
        let mut repr = format!("TreeNode(id={:?}, text={:?}", n.id.to_string(), n.text);
        if let Some(kind) = &n.kind {
            let _ = write!(repr, ", kind={kind:?}");
        }
        repr.push(')');
        repr
    }
}

/// Consumption series of one utility.
#[pyclass(frozen, eq, module = "provident")]
#[derive(PartialEq)]
pub struct Series(pub quick_graphs::Series);

#[pymethods]
impl Series {
    /// Utility with its unit, as a display label: `Cold Water (m3)`.
    #[getter]
    fn utility(&self) -> &str {
        &self.0.utility
    }

    /// Unit of measure, e.g. `m3` or `kWh`.
    #[getter]
    fn units(&self) -> &str {
        &self.0.units
    }

    /// `<suite>|<group>|<site>`; just `<suite>` when aggregated.
    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }

    /// Site (building) name.
    #[getter]
    fn site(&self) -> &str {
        &self.0.site
    }

    /// Consumption per interval (hourly up to 7 days, daily beyond), oldest first.
    #[getter]
    fn data(&self) -> Vec<f64> {
        self.0.data.clone()
    }

    fn __repr__(&self) -> String {
        let s = &self.0;
        format!(
            "Series(utility={:?}, name={:?}, points={})",
            s.utility,
            s.name,
            s.data.len()
        )
    }
}

/// One row of a usage report: readings and consumption for a meter or aggregated group.
#[pyclass(frozen, eq, module = "provident")]
#[derive(PartialEq)]
pub struct ReportRow(pub quick_report::ReportRow);

#[pymethods]
impl ReportRow {
    /// Suite/unit name.
    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }

    /// Meter serial; empty for aggregated rows.
    #[getter]
    fn serial(&self) -> &str {
        &self.0.serial
    }

    /// Meter reading at the end of the range.
    #[getter]
    fn current(&self) -> Option<f64> {
        self.0.current
    }

    /// Meter reading at the start of the range.
    #[getter]
    fn previous(&self) -> Option<f64> {
        self.0.previous
    }

    /// Consumption over the range (`current - previous`).
    #[getter]
    fn total(&self) -> Option<f64> {
        self.0.total
    }

    /// Average consumption per day over the range.
    #[getter]
    fn daily_average(&self) -> Option<f64> {
        self.0.daily_average
    }

    /// Readings are missing or incomplete for the range; values may be `None`.
    #[getter]
    fn error(&self) -> bool {
        self.0.error
    }

    /// Meter id, for a per-meter row.
    #[getter]
    fn meter_id(&self) -> Option<u64> {
        match self.0.source() {
            RowSource::Meter(id) => Some(id.0),
            RowSource::Group(_) => None,
        }
    }

    /// Group id, for an aggregated row (`aggregate_groups=True`).
    #[getter]
    fn group_id(&self) -> Option<u64> {
        match self.0.source() {
            RowSource::Group(id) => Some(id.0),
            RowSource::Meter(_) => None,
        }
    }

    fn __repr__(&self) -> String {
        let r = &self.0;
        format!(
            "ReportRow(name={:?}, serial={:?}, total={}, error={})",
            r.name,
            r.serial,
            r.total.map_or("None".to_owned(), |t| t.to_string()),
            if r.error { "True" } else { "False" }
        )
    }
}

/// Profile of the signed-in account.
#[pyclass(frozen, eq, module = "provident")]
#[derive(PartialEq)]
pub struct AccountDetails(pub get_details::AccountDetails);

#[pymethods]
impl AccountDetails {
    /// Login name.
    #[getter]
    fn login(&self) -> &str {
        &self.0.login
    }

    /// Given name.
    #[getter]
    fn first_name(&self) -> &str {
        &self.0.first_name
    }

    /// Family name.
    #[getter]
    fn last_name(&self) -> &str {
        &self.0.last_name
    }

    /// Contact email address, if any.
    #[getter]
    fn email(&self) -> Option<&str> {
        self.0.email.as_deref()
    }

    /// Names of the groups (suites/units) the account belongs to.
    #[getter]
    fn groups(&self) -> Vec<String> {
        self.0.groups.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "AccountDetails(login={:?}, groups={:?})",
            self.0.login, self.0.groups
        )
    }
}

/// One current/previous pair from the published rates page (rate plus effective dates).
#[pyclass(frozen, eq, module = "provident")]
#[derive(PartialEq)]
pub struct RateEntry(pub parsed::RateEntry);

#[pymethods]
impl RateEntry {
    /// When the current rate took effect, if shown.
    #[getter]
    fn current_effective(&self) -> Option<NaiveDate> {
        self.0.current_effective
    }

    /// Current rate, if shown.
    #[getter]
    fn current(&self) -> Option<f64> {
        self.0.current
    }

    /// When the previous rate took effect, if shown.
    #[getter]
    fn previous_effective(&self) -> Option<NaiveDate> {
        self.0.previous_effective
    }

    /// Previous rate, if shown.
    #[getter]
    fn previous(&self) -> Option<f64> {
        self.0.previous
    }

    /// The current rate, falling back to the previous one.
    fn value(&self) -> Option<f64> {
        self.0.value()
    }

    fn __repr__(&self) -> String {
        format!(
            "RateEntry(current={}, previous={:?})",
            {
                let e = &self.0;
                e.current.map_or("None".to_owned(), |v| v.to_string())
            },
            self.0.previous
        )
    }
}

impl RateEntry {
    /// Wraps an entry, for getters of [`PublishedRates`].
    fn wrap(entry: Option<parsed::RateEntry>) -> Option<Self> {
        entry.map(Self)
    }
}

/// Rates and fees Provident publishes for the account's location.
#[pyclass(frozen, eq, module = "provident")]
#[derive(PartialEq)]
pub struct PublishedRates(pub parsed::Rates);

#[pymethods]
impl PublishedRates {
    /// Tiered plan, first tier (¢/kWh below the seasonal threshold).
    #[getter]
    fn tier_1(&self) -> Option<RateEntry> {
        RateEntry::wrap(self.0.electricity.tiered.get(&parsed::Tier::First).copied())
    }

    /// Tiered plan, second tier (¢/kWh above the threshold).
    #[getter]
    fn tier_2(&self) -> Option<RateEntry> {
        RateEntry::wrap(
            self.0
                .electricity
                .tiered
                .get(&parsed::Tier::Second)
                .copied(),
        )
    }

    /// Time-of-use off-peak rate (¢/kWh).
    #[getter]
    fn tou_off_peak(&self) -> Option<RateEntry> {
        RateEntry::wrap(
            self.0
                .electricity
                .tou
                .get(&parsed::TouPeriod::OffPeak)
                .copied(),
        )
    }

    /// Time-of-use mid-peak rate (¢/kWh).
    #[getter]
    fn tou_mid_peak(&self) -> Option<RateEntry> {
        RateEntry::wrap(
            self.0
                .electricity
                .tou
                .get(&parsed::TouPeriod::MidPeak)
                .copied(),
        )
    }

    /// Time-of-use on-peak rate (¢/kWh).
    #[getter]
    fn tou_on_peak(&self) -> Option<RateEntry> {
        RateEntry::wrap(
            self.0
                .electricity
                .tou
                .get(&parsed::TouPeriod::OnPeak)
                .copied(),
        )
    }

    /// Ultra-low overnight rate (¢/kWh), every night.
    #[getter]
    fn ulo_ultra_low_overnight(&self) -> Option<RateEntry> {
        RateEntry::wrap(
            self.0
                .electricity
                .ulo
                .get(&parsed::UloPeriod::UltraLowOvernight)
                .copied(),
        )
    }

    /// ULO weekend off-peak rate (¢/kWh).
    #[getter]
    fn ulo_weekend_off_peak(&self) -> Option<RateEntry> {
        RateEntry::wrap(
            self.0
                .electricity
                .ulo
                .get(&parsed::UloPeriod::WeekendOffPeak)
                .copied(),
        )
    }

    /// ULO mid-peak rate (¢/kWh).
    #[getter]
    fn ulo_mid_peak(&self) -> Option<RateEntry> {
        RateEntry::wrap(
            self.0
                .electricity
                .ulo
                .get(&parsed::UloPeriod::MidPeak)
                .copied(),
        )
    }

    /// ULO on-peak rate (¢/kWh).
    #[getter]
    fn ulo_on_peak(&self) -> Option<RateEntry> {
        RateEntry::wrap(
            self.0
                .electricity
                .ulo
                .get(&parsed::UloPeriod::OnPeak)
                .copied(),
        )
    }

    /// Provident's administration fee per 30 days, in currency units.
    #[getter]
    fn admin_fee(&self) -> Option<RateEntry> {
        self.0.fees.admin.map(RateEntry)
    }

    /// Provident's meter fee per 30 days, in currency units.
    #[getter]
    fn meter_fee(&self) -> Option<RateEntry> {
        self.0.fees.meter.map(RateEntry)
    }

    /// Rows the parser could not classify: label -> raw cell texts.
    #[getter]
    fn unknown(&self) -> BTreeMap<String, Vec<String>> {
        self.0.unknown.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "PublishedRates(tou_on_peak={}, ulo_on_peak={}, admin_fee={})",
            self.tou_on_peak().map_or("None".to_owned(), |e| e
                .value()
                .map_or("None".to_owned(), |v| v.to_string())),
            self.ulo_on_peak().map_or("None".to_owned(), |e| e
                .value()
                .map_or("None".to_owned(), |v| v.to_string())),
            self.admin_fee().map_or("None".to_owned(), |e| e
                .value()
                .map_or("None".to_owned(), |v| v.to_string()))
        )
    }
}

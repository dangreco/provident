// SPDX-License-Identifier: GPL-3.0-only

//! Rates and fees parsed from the rates page.
//!
//! Parsing is total and conservative: a restyled page yields fewer fields, never wrong ones
//! and never an error. Unrecognized or implausible rows are kept in [`Rates::unknown`] for
//! diagnostics. The rules (and the edits each one survives) are documented on the parser,
//! `rates_and_fees::parse`.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use scraper::Html;

use super::parse;

/// A current/previous pair: the effective date and value of each, any of which may be absent.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RateEntry {
    /// When the current value took effect, if shown.
    pub current_effective: Option<NaiveDate>,
    /// Current value, if shown.
    pub current: Option<f64>,
    /// When the previous value took effect, if shown.
    pub previous_effective: Option<NaiveDate>,
    /// Previous value, if shown.
    pub previous: Option<f64>,
}

impl RateEntry {
    /// The current value, falling back to the previous one.
    pub fn value(&self) -> Option<f64> {
        self.current.or(self.previous)
    }
}

/// Electricity energy rates, in cents per kWh, by plan and period.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ElectricityRates {
    /// Tiered pricing: usage below/above the seasonal threshold.
    pub tiered: BTreeMap<Tier, RateEntry>,
    /// Time-of-use pricing: off/mid/on-peak.
    pub tou: BTreeMap<TouPeriod, RateEntry>,
    /// Ultra-low overnight pricing: overnight/weekend off/mid/on-peak.
    pub ulo: BTreeMap<UloPeriod, RateEntry>,
}

/// Provident's own fixed charges.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Fees {
    /// Monthly administration fee (per 30 days), in currency units.
    pub admin: Option<RateEntry>,
    /// Monthly meter fee (per 30 days), in currency units.
    pub meter: Option<RateEntry>,
}

/// Everything the rates page publishes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rates {
    /// Electricity energy rates (¢/kWh).
    pub electricity: ElectricityRates,
    /// Provident's administration and delivery charges.
    pub fees: Fees,
    /// Tables for anything other than suite electricity (EV charging, water, gas, thermal, …),
    /// keyed by section heading, then row label. Their layout and units are not known, so rows
    /// are kept as published; they never feed [`Rates::electricity`].
    pub sections: BTreeMap<String, BTreeMap<String, RateEntry>>,
    /// Labels of rows the parser could not classify, with their raw cell texts.
    pub unknown: BTreeMap<String, Vec<String>>,
}

/// Tiered-plan tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// Lower price up to the seasonal threshold.
    First,
    /// Higher price above the threshold.
    Second,
}

/// Time-of-use price periods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TouPeriod {
    /// Weeknights, weekends, and holidays.
    OffPeak,
    /// Shoulder hours.
    MidPeak,
    /// Highest-price hours.
    OnPeak,
}

/// Ultra-low-overnight price periods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UloPeriod {
    /// Every night 11 p.m.–7 a.m.
    UltraLowOvernight,
    /// Weekend and holiday days.
    WeekendOffPeak,
    /// Weekday shoulder hours.
    MidPeak,
    /// Weekday late afternoon/evening.
    OnPeak,
}

impl Rates {
    /// Every recognizable rate in the page's HTML. Never fails: anything unrecognized is
    /// skipped or kept in [`Rates::unknown`].
    pub fn parse(html: &str) -> Self {
        parse::rates(&Html::parse_document(html))
    }

    /// Like [`Rates::parse`], but `None` when the HTML is the site's sign-in form (no or
    /// expired session) rather than a rates page.
    pub fn parse_page(html: &str) -> Option<Self> {
        parse::page(&Html::parse_document(html))
    }

    /// `true` when no rate or fee was recognized.
    pub fn is_empty(&self) -> bool {
        let e = &self.electricity;
        e.tiered.is_empty()
            && e.tou.is_empty()
            && e.ulo.is_empty()
            && self.fees.admin.is_none()
            && self.fees.meter.is_none()
            && self.sections.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = include_str!("../../../../tests/fixtures/rates_page.html");

    #[test]
    fn parses_the_observed_page() {
        let rates = Rates::parse(PAGE);
        let tou = &rates.electricity.tou;
        assert_eq!(
            tou.get(&TouPeriod::OffPeak).map(|r| r.current),
            Some(Some(9.8)),
            "{rates:?}"
        );
        assert_eq!(
            tou.get(&TouPeriod::OnPeak).map(|r| r.previous),
            Some(Some(15.8))
        );
        let ulo = &rates.electricity.ulo;
        assert_eq!(
            ulo.get(&UloPeriod::UltraLowOvernight).map(|r| r.current),
            Some(Some(3.9))
        );
        assert_eq!(
            rates
                .electricity
                .tiered
                .get(&Tier::Second)
                .and_then(|r| r.current),
            Some(14.2)
        );
        assert_eq!(
            rates.fees.admin.as_ref().and_then(|f| f.current),
            Some(11.90)
        );
        assert_eq!(
            rates.fees.meter.as_ref().and_then(|f| f.previous),
            Some(8.31)
        );
        assert!(rates.unknown.is_empty(), "{:?}", rates.unknown);
    }

    #[test]
    fn empty_and_foreign_html_yield_defaults() {
        assert_eq!(Rates::parse(""), Rates::default());
        assert_eq!(
            Rates::parse("<html><body><p>hi</p></body></html>"),
            Rates::default()
        );
    }

    #[test]
    fn restyled_or_partial_rows_do_not_fail() {
        // Same labels, no dates, dashes for missing values, extra unknown row.
        let html = r#"
        <div class="rates-holder">
        <h2>Electricity Energy Rates</h2>
        <table class="rates1"><tr><th></th><th>Effective Date</th><th>Rate</th><th>Effective Date</th><th>Rate</th></tr>
        <tr><td>Time-Of-Use (TOU)</td><td></td><td></td><td></td><td></td></tr>
        <tr><td>Off-Peak</td><td>—</td><td>9.8</td><td></td><td>—</td></tr>
        <tr><td>Surcharge</td><td>x</td><td>1</td><td>y</td><td>2</td></tr>
        </table>
        <h2>Provident’s Administration and Delivery Charges</h2>
        <table class="rates1 rates2"></table>
        </div>"#;
        let rates = Rates::parse(html);
        assert_eq!(
            rates
                .electricity
                .tou
                .get(&TouPeriod::OffPeak)
                .map(|r| r.current),
            Some(Some(9.8))
        );
        assert!(
            rates
                .electricity
                .tou
                .get(&TouPeriod::OffPeak)
                .unwrap()
                .previous
                .is_none()
        );
        assert!(rates.fees.admin.is_none());
        assert_eq!(rates.unknown.keys().collect::<Vec<_>>(), ["Surcharge"]);
    }
}

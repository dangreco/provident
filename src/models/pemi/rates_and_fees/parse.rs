// SPDX-License-Identifier: GPL-3.0-only

//! HTML parsing for the rates page, behind [`super::parsed::Rates::parse`].
//!
//! The page has been stable, but it is hand-edited CMS markup, so nothing here trusts layout
//! alone. Each rule below exists so a plausible edit degrades to "fewer fields" rather than
//! "wrong numbers":
//!
//! - **Tables, not headings, are the unit.** Every `<table>` is read on its own; its section is
//!   its `<caption>` or else the nearest preceding `h1`–`h6`. Headings are optional: labels
//!   alone identify electricity rows and fee rows.
//! - **Other sections never feed electricity.** A table whose section names EV charging or
//!   another utility (water, gas, thermal, …) is kept in [`Rates::sections`] under its heading,
//!   so its own "On-Peak" cannot replace the suite electricity one, wherever it sits on the
//!   page. A fees section only yields fees.
//! - **Plans are scoped, then inferred.** A subhead row (label with every other cell empty)
//!   naming a plan scopes the rows below it. Without subheads, plan-unique labels (`Tier 1`,
//!   `Ultra-Low Overnight`, `Weekend Off-Peak`) set the plan, and the ambiguous `Mid-Peak` /
//!   `On-Peak` follow it (ULO after a ULO-only label, TOU otherwise).
//! - **Labels are compared compacted.** Case, spacing, hyphens, and punctuation are ignored:
//!   `Off-Peak`, `OFF PEAK`, and `Offpeak` are the same row.
//! - **Cells are typed by content.** Each current/previous pair is read as "the cell that is a
//!   date" plus "the cell that is a number", so swapped columns still parse. Any other column
//!   count falls back to scanning the cells in order.
//! - **Numbers are the first numeric token.** `$11.90`, `9.8¢`, `1,234.5`, and `12-15` give
//!   11.9, 9.8, 1234.5, and 12. Dates are never numbers.
//! - **Values must be plausible.** Energy rates must fall in (0, 500] ¢/kWh and fees in
//!   (0, 1000]. Anything else is dropped and the row is kept in [`Rates::unknown`].
//! - **First row wins.** A duplicate label never replaces an earlier one.
//! - **The login form is not "no rates".** [`page`] reports it as `None` so callers can sign in
//!   again instead of treating an expired session as rates that vanished.

use std::sync::LazyLock;

use chrono::NaiveDate;
use scraper::{ElementRef, Html, Selector};

use super::parsed::{RateEntry, Rates, Tier, TouPeriod, UloPeriod};

fn selector(css: &str) -> Selector {
    Selector::parse(css).expect("valid selector")
}

/// Section headings and tables in document order (captions are read per table).
static HEADINGS_AND_TABLES: LazyLock<Selector> =
    LazyLock::new(|| selector("h1, h2, h3, h4, h5, h6, table"));
static CAPTION: LazyLock<Selector> = LazyLock::new(|| selector("caption"));
static ROWS: LazyLock<Selector> = LazyLock::new(|| selector("tr"));
static CELLS: LazyLock<Selector> = LazyLock::new(|| selector("th, td"));
/// The sign-in form the site serves in place of the rates when there is no session.
static LOGIN_FORM: LazyLock<Selector> =
    LazyLock::new(|| selector(r#"form[action*="rates-login"]"#));

/// Largest plausible energy rate, in ¢/kWh (the highest Ontario RPP price is ~40).
const MAX_ENERGY_CENTS: f64 = 500.0;
/// Largest plausible monthly Provident fee, in currency units.
const MAX_FEE: f64 = 1000.0;

/// Words that mark a section as another utility's, whose rows must not be read as electricity.
const FOREIGN_SECTIONS: &[&str] = &[
    "water", "gas", "thermal", "heating", "cooling", "sewer", "steam",
];

/// What a table's section heading says it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Section {
    /// Suite electricity, or no telling heading: energy rates and fees by label.
    Electricity,
    /// Provident's own charges: fees only.
    Fees,
    /// EV charging or another utility, kept apart under its heading.
    Other(String),
}

fn section(heading: &str) -> Section {
    let words = words(heading);
    let starts = |prefix: &str| words.iter().any(|w| w.starts_with(prefix));
    let ev = words.iter().any(|w| w == "ev" || w == "evs")
        || starts("vehicle")
        || starts("charging")
        || starts("charger");
    if ev || FOREIGN_SECTIONS.iter().any(|f| starts(f)) {
        return Section::Other(heading.to_owned());
    }
    let energy = starts("electric") || starts("energy") || words.iter().any(|w| w == "rates");
    let fees = starts("administration") || starts("fee") || starts("delivery");
    if fees && !energy {
        Section::Fees
    } else {
        Section::Electricity
    }
}

/// The page's rates, or `None` when it is the sign-in form (no or expired session).
pub(crate) fn page(document: &Html) -> Option<Rates> {
    let rates = rates(document);
    let login_form = document.select(&LOGIN_FORM).next().is_some();
    (!(login_form && rates.is_empty())).then_some(rates)
}

/// Every recognizable rate on the page; never fails.
pub(crate) fn rates(document: &Html) -> Rates {
    let mut rates = Rates::default();
    let mut heading = String::new();
    for node in document.select(&HEADINGS_AND_TABLES) {
        if node.value().name() != "table" {
            heading = cell_text(node);
            continue;
        }
        let caption = node.select(&CAPTION).next().map(cell_text);
        let section_heading = caption.as_deref().unwrap_or(&heading);
        read_table(node, &section(section_heading), &mut rates);
    }
    rates
}

/// Which plan a row belongs to, while walking a table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Plan {
    Tiered,
    Tou,
    Ulo,
}

/// Where a data row's values go.
enum Destination {
    Tier(Tier),
    Tou(TouPeriod),
    Ulo(UloPeriod),
    Admin,
    Meter,
}

fn read_table(table: ElementRef<'_>, section: &Section, rates: &mut Rates) {
    let mut plan = None;
    for tr in table.select(&ROWS) {
        let cells: Vec<String> = tr.select(&CELLS).map(cell_text).collect();
        let Some((label, rest)) = cells.split_first() else {
            continue;
        };
        let compact = compact(label);
        if compact.is_empty() {
            // Column headers ("", "Effective Date", "Rate") and spacer rows.
            continue;
        }
        if rest.iter().all(|c| c.is_empty()) {
            if let Some(p) = plan_subhead(label) {
                plan = Some(p);
            }
            continue;
        }
        let entry = entry(rest);
        if entry.current.is_none() && entry.previous.is_none() {
            // Header-like rows with text but no numbers.
            continue;
        }
        if let Section::Other(name) = section {
            // No layout assumptions for other sections: keep the row as published.
            let stored = plausible(entry, f64::MAX).is_some_and(|entry| {
                let rows = rates.sections.entry(name.clone()).or_default();
                match rows.entry(label.clone()) {
                    std::collections::btree_map::Entry::Vacant(v) => {
                        v.insert(entry);
                        true
                    }
                    std::collections::btree_map::Entry::Occupied(_) => false,
                }
            });
            if !stored {
                rates
                    .unknown
                    .entry(label.clone())
                    .or_insert_with(|| cells.clone());
            }
            continue;
        }
        let destination = destination(&compact, &mut plan).filter(|d| {
            *section != Section::Fees || matches!(d, Destination::Admin | Destination::Meter)
        });
        let stored = match destination {
            Some(Destination::Tier(tier)) => store(
                rates.electricity.tiered.entry(tier),
                entry,
                MAX_ENERGY_CENTS,
            ),
            Some(Destination::Tou(period)) => {
                store(rates.electricity.tou.entry(period), entry, MAX_ENERGY_CENTS)
            }
            Some(Destination::Ulo(period)) => {
                store(rates.electricity.ulo.entry(period), entry, MAX_ENERGY_CENTS)
            }
            Some(Destination::Admin) => store_fee(&mut rates.fees.admin, entry),
            Some(Destination::Meter) => store_fee(&mut rates.fees.meter, entry),
            None => false,
        };
        if !stored {
            rates
                .unknown
                .entry(label.clone())
                .or_insert_with(|| cells.clone());
        }
    }
}

/// Inserts a plausible `entry` if the slot is empty; `false` when nothing was stored.
fn store<K: Ord>(
    slot: std::collections::btree_map::Entry<'_, K, RateEntry>,
    entry: RateEntry,
    max: f64,
) -> bool {
    let Some(entry) = plausible(entry, max) else {
        return false;
    };
    match slot {
        std::collections::btree_map::Entry::Vacant(v) => {
            v.insert(entry);
            true
        }
        std::collections::btree_map::Entry::Occupied(_) => false,
    }
}

fn store_fee(slot: &mut Option<RateEntry>, entry: RateEntry) -> bool {
    if slot.is_some() {
        return false;
    }
    match plausible(entry, MAX_FEE) {
        Some(entry) => {
            *slot = Some(entry);
            true
        }
        None => false,
    }
}

/// `entry` with out-of-range values removed; `None` when no value survives.
fn plausible(entry: RateEntry, max: f64) -> Option<RateEntry> {
    let ok = |v: &f64| v.is_finite() && *v > 0.0 && *v <= max;
    let entry = RateEntry {
        current: entry.current.filter(ok),
        previous: entry.previous.filter(ok),
        ..entry
    };
    (entry.current.is_some() || entry.previous.is_some()).then_some(entry)
}

/// Maps a compacted row label to its destination, updating the inferred plan.
fn destination(compact: &str, plan: &mut Option<Plan>) -> Option<Destination> {
    let ulo = *plan == Some(Plan::Ulo);
    let mut set = |p| {
        *plan = Some(p);
    };
    Some(match compact {
        "tier1" | "tierone" | "tieri" => {
            set(Plan::Tiered);
            Destination::Tier(Tier::First)
        }
        "tier2" | "tiertwo" | "tierii" => {
            set(Plan::Tiered);
            Destination::Tier(Tier::Second)
        }
        "ultralowovernight" | "ultralow" => {
            set(Plan::Ulo);
            Destination::Ulo(UloPeriod::UltraLowOvernight)
        }
        "weekendoffpeak" | "weekendandholidayoffpeak" => {
            set(Plan::Ulo);
            Destination::Ulo(UloPeriod::WeekendOffPeak)
        }
        // ULO has no plain off-peak; under ULO it is ambiguous, so keep it unknown.
        "offpeak" if ulo => return None,
        "offpeak" => {
            set(Plan::Tou);
            Destination::Tou(TouPeriod::OffPeak)
        }
        "midpeak" if ulo => Destination::Ulo(UloPeriod::MidPeak),
        "midpeak" => {
            set(Plan::Tou);
            Destination::Tou(TouPeriod::MidPeak)
        }
        "onpeak" if ulo => Destination::Ulo(UloPeriod::OnPeak),
        "onpeak" => {
            set(Plan::Tou);
            Destination::Tou(TouPeriod::OnPeak)
        }
        c if c.contains("administration") || c.contains("adminfee") => Destination::Admin,
        c if c.starts_with("meter") && (c.contains("fee") || c.contains("charge")) => {
            Destination::Meter
        }
        _ => return None,
    })
}

/// The plan a subhead row names (`Tiered`, `Time-Of-Use (TOU)`, `Ultra-Low Overnight`, …).
fn plan_subhead(label: &str) -> Option<Plan> {
    let words = words(label);
    let has = |w: &str| words.iter().any(|x| x == w);
    let joined = words.join(" ");
    if joined.contains("ultra low") || has("ulo") {
        Some(Plan::Ulo)
    } else if joined.contains("time of use") || has("tou") {
        Some(Plan::Tou)
    } else if words.iter().any(|w| w.starts_with("tier")) {
        Some(Plan::Tiered)
    } else {
        None
    }
}

/// Current/previous pair from the cells after the label.
fn entry(rest: &[String]) -> RateEntry {
    let (current_effective, current, previous_effective, previous) = match rest {
        // The page's layout: effective date + rate, current then previous (either order
        // within a pair).
        [a, b, c, d] => {
            let (cd, cv) = pair(a, b);
            let (pd, pv) = pair(c, d);
            (cd, cv, pd, pv)
        }
        [a, b] => {
            let (cd, cv) = pair(a, b);
            (cd, cv, None, None)
        }
        cells => {
            let mut dates = cells.iter().filter_map(|c| date(c));
            let mut values = cells
                .iter()
                .filter(|c| date(c).is_none())
                .filter_map(|c| number(c));
            (dates.next(), values.next(), dates.next(), values.next())
        }
    };
    RateEntry {
        current_effective,
        current,
        previous_effective,
        previous,
    }
}

/// `(date, value)` from two cells in either order.
fn pair(a: &str, b: &str) -> (Option<NaiveDate>, Option<f64>) {
    match (date(a), date(b)) {
        (Some(d), _) => (Some(d), number(b)),
        (None, Some(d)) => (Some(d), number(a)),
        (None, None) => (None, number(b).or_else(|| number(a))),
    }
}

/// Whitespace-squeezed text of `element`; NBSPs count as whitespace.
fn cell_text(element: ElementRef<'_>) -> String {
    element
        .text()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lowercase alphanumeric words of `text` (`"Time-Of-Use (TOU)"` → `[time, of, use, tou]`).
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// [`words`] joined without separators (`"Off-Peak"` → `offpeak`).
fn compact(text: &str) -> String {
    words(text).concat()
}

/// `Some(date)` for ISO (`2025-11-01`, `2025/11/01`) or English (`Nov 1, 2025`,
/// `November 1, 2025`, `1 November 2025`) dates; anything else is `None`, not an error.
fn date(text: &str) -> Option<NaiveDate> {
    let text = text.trim().replace('.', "");
    if text.is_empty() {
        return None;
    }
    [
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%b %d, %Y",
        "%B %d, %Y",
        "%d %B %Y",
        "%d %b %Y",
    ]
    .iter()
    .find_map(|f| NaiveDate::parse_from_str(&text, f).ok())
}

/// The first number in `text`: `$11.90` → 11.9, `9.8¢` → 9.8, `1,234.5` → 1234.5,
/// `12-15` → 12. Dates, dashes, and words are `None`.
fn number(text: &str) -> Option<f64> {
    if date(text).is_some() {
        return None;
    }
    let start = text.char_indices().find_map(|(i, c)| {
        let leading_dot = c == '.' && text[i + 1..].starts_with(|n: char| n.is_ascii_digit());
        (c.is_ascii_digit() || leading_dot).then_some(i)
    })?;
    let mut digits = String::new();
    let mut seen_dot = false;
    for c in text[start..].chars() {
        match c {
            '0'..='9' => digits.push(c),
            // Thousands separator, only before the decimal point.
            ',' if !seen_dot => {}
            '.' if !seen_dot => {
                seen_dot = true;
                digits.push('.');
            }
            _ => break,
        }
    }
    digits.parse::<f64>().ok().filter(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(html: &str) -> Rates {
        rates(&Html::parse_document(html))
    }

    fn tou(rates: &Rates, period: TouPeriod) -> Option<f64> {
        rates.electricity.tou.get(&period).and_then(|e| e.current)
    }

    fn ulo(rates: &Rates, period: UloPeriod) -> Option<f64> {
        rates.electricity.ulo.get(&period).and_then(|e| e.current)
    }

    #[test]
    fn numbers_take_the_first_numeric_token() {
        assert_eq!(number("$11.90"), Some(11.9));
        assert_eq!(number("9.8¢"), Some(9.8));
        assert_eq!(number("1,234.5"), Some(1234.5));
        assert_eq!(number("12-15"), Some(12.0));
        assert_eq!(number(".5"), Some(0.5));
        assert_eq!(number("12"), Some(12.0));
        for missing in ["", "—", "-", "N/A", "TBD"] {
            assert_eq!(number(missing), None, "{missing:?}");
        }
        // A date is never a number (it would read as 20251101).
        assert_eq!(number("2025-11-01"), None);
    }

    #[test]
    fn dates_accept_iso_and_english() {
        let nov1 = NaiveDate::from_ymd_opt(2025, 11, 1);
        for text in [
            "2025-11-01",
            "2025/11/01",
            "Nov 1, 2025",
            "Nov. 1, 2025",
            "November 1, 2025",
            "1 November 2025",
            " 2025-11-01 ",
        ] {
            assert_eq!(date(text), nov1, "{text:?}");
        }
        for text in ["", "—", "12", "Current"] {
            assert_eq!(date(text), None, "{text:?}");
        }
    }

    #[test]
    fn pairs_tolerate_swapped_and_missing_cells() {
        let nov1 = NaiveDate::from_ymd_opt(2025, 11, 1);
        assert_eq!(pair("2025-11-01", "9.8"), (nov1, Some(9.8)));
        assert_eq!(pair("9.8", "2025-11-01"), (nov1, Some(9.8)));
        assert_eq!(pair("—", "9.8"), (None, Some(9.8)));
        assert_eq!(pair("9.8", ""), (None, Some(9.8)));
        assert_eq!(pair("", ""), (None, None));
    }

    #[test]
    fn restyled_page_still_parses() {
        // h3 + caption instead of h2, no plan subheads, uppercase/odd spacing, swapped
        // columns, NBSPs, a missing previous value.
        let html = "
            <h3>Energy</h3>
            <table>
              <caption>Electricity Energy Rates</caption>
              <tr><td>TIER 1</td><td>12</td><td>2025-11-01</td><td>9.3</td><td>2024-11-01</td></tr>
              <tr><td>Tier&nbsp;2</td><td>2025-11-01</td><td>14.2</td><td>—</td><td>—</td></tr>
              <tr><td>Off Peak</td><td>2025-11-01</td><td>9.8</td><td>2024-11-01</td><td>7.6</td></tr>
              <tr><td>Mid-Peak</td><td>2025-11-01</td><td>15.7</td><td>2024-11-01</td><td>12.2</td></tr>
              <tr><td>On-Peak</td><td>2025-11-01</td><td>20.3</td><td>2024-11-01</td><td>15.8</td></tr>
              <tr><td>Ultra-Low-Overnight</td><td>2025-11-01</td><td>3.9</td><td>2024-11-01</td><td>2.8</td></tr>
              <tr><td>Weekend Off-Peak</td><td>2025-11-01</td><td>9.8</td><td>2024-11-01</td><td>7.6</td></tr>
              <tr><td>Mid-Peak</td><td>2025-11-01</td><td>15.7</td><td>2024-11-01</td><td>12.2</td></tr>
              <tr><td>On-Peak</td><td>2025-11-01</td><td>39.1</td><td>2024-11-01</td><td>28.4</td></tr>
            </table>
            <h4>Fees</h4>
            <table>
              <tr><td>Electricity Administration Fee</td><td>2026-04-16</td><td>$11.90</td><td>2025-11-16</td><td>$11.69</td></tr>
              <tr><td>Meter Fee</td><td>2026-04-16</td><td>$8.46</td><td>2025-11-16</td><td>$8.31</td></tr>
            </table>";
        let rates = parse(html);
        let tiers = &rates.electricity.tiered;
        assert_eq!(tiers[&Tier::First].current, Some(12.0));
        assert_eq!(
            tiers[&Tier::First].current_effective,
            NaiveDate::from_ymd_opt(2025, 11, 1)
        );
        assert_eq!(tiers[&Tier::Second].previous, None);
        assert_eq!(tou(&rates, TouPeriod::OnPeak), Some(20.3));
        // Inferred from the ULO-only rows above them, without subheads.
        assert_eq!(ulo(&rates, UloPeriod::OnPeak), Some(39.1));
        assert_eq!(ulo(&rates, UloPeriod::MidPeak), Some(15.7));
        assert_eq!(rates.fees.admin.and_then(|f| f.current), Some(11.9));
        assert_eq!(rates.fees.meter.and_then(|f| f.previous), Some(8.31));
        assert!(rates.unknown.is_empty(), "{:?}", rates.unknown);
    }

    #[test]
    fn subheads_scope_ambiguous_labels() {
        let html = "
            <table>
              <tr><td>Ultra-Low Overnight (ULO)</td><td></td><td></td><td></td><td></td></tr>
              <tr><td>On-Peak</td><td>2025-11-01</td><td>39.1</td><td></td><td></td></tr>
              <tr><td>Time-of-Use</td><td></td><td></td><td></td><td></td></tr>
              <tr><td>On-Peak</td><td>2025-11-01</td><td>20.3</td><td></td><td></td></tr>
            </table>";
        let rates = parse(html);
        assert_eq!(ulo(&rates, UloPeriod::OnPeak), Some(39.1));
        assert_eq!(tou(&rates, TouPeriod::OnPeak), Some(20.3));
    }

    #[test]
    fn other_sections_are_kept_apart_wherever_they_sit() {
        // EV and water tables before *and* after electricity; same labels as electricity's.
        let html = "
            <h2>EV Charging Rates</h2>
            <table><tr><td>Time-Of-Use (TOU)</td><td></td></tr>
              <tr><td>On-Peak</td><td>2025-11-01</td><td>25.0</td><td>2024-11-01</td><td>22.0</td></tr>
              <tr><td>Off-Peak</td><td>2025-11-01</td><td>8.0</td><td></td><td></td></tr></table>
            <h2>Electricity Energy Rates</h2>
            <table><tr><td>Time-Of-Use (TOU)</td><td></td></tr>
              <tr><td>On-Peak</td><td>2025-11-01</td><td>20.3</td><td></td><td></td></tr></table>
            <h2>Hot Water Rates</h2>
            <table><tr><td>Rate per m³</td><td>2025-11-01</td><td>4.2</td><td></td><td></td></tr></table>";
        let rates = parse(html);
        assert_eq!(tou(&rates, TouPeriod::OnPeak), Some(20.3));
        assert_eq!(tou(&rates, TouPeriod::OffPeak), None);
        let ev = &rates.sections["EV Charging Rates"];
        assert_eq!(ev["On-Peak"].current, Some(25.0));
        assert_eq!(ev["On-Peak"].previous, Some(22.0));
        assert_eq!(ev["Off-Peak"].current, Some(8.0));
        assert_eq!(
            rates.sections["Hot Water Rates"]["Rate per m³"].current,
            Some(4.2)
        );
        assert!(rates.unknown.is_empty(), "{:?}", rates.unknown);
    }

    #[test]
    fn section_headings_are_classified() {
        for other in [
            "EV Charging Rates",
            "Electric Vehicle Charging",
            "EV Charger Rates",
            "Cold Water",
            "Gas Rates",
            "Thermal Energy (Heating)",
        ] {
            assert_eq!(section(other), Section::Other(other.into()), "{other}");
        }
        for electricity in ["Electricity Energy Rates", "Energy Rates", "", "Rates"] {
            assert_eq!(
                section(electricity),
                Section::Electricity,
                "{electricity:?}"
            );
        }
        assert_eq!(
            section("Provident’s Administration and Delivery Charges"),
            Section::Fees
        );
    }

    #[test]
    fn fees_section_never_yields_energy_rates() {
        let html = "
            <h2>Administration and Delivery Charges</h2>
            <table>
              <tr><td>On-Peak</td><td>2026-04-16</td><td>$3.00</td><td></td><td></td></tr>
              <tr><td>Meter Fee</td><td>2026-04-16</td><td>$8.46</td><td></td><td></td></tr>
            </table>";
        let rates = parse(html);
        assert!(rates.electricity.tou.is_empty(), "{rates:?}");
        assert_eq!(rates.fees.meter.and_then(|f| f.current), Some(8.46));
        assert!(rates.unknown.contains_key("On-Peak"));
    }

    #[test]
    fn implausible_and_duplicate_rows_are_kept_aside() {
        let html = "
            <table>
              <tr><td>Off-Peak</td><td>2025-11-01</td><td>9.8</td><td></td><td></td></tr>
              <tr><td>Off-Peak</td><td>2025-11-01</td><td>1.1</td><td></td><td></td></tr>
              <tr><td>Mid-Peak</td><td>2025-11-01</td><td>15700</td><td></td><td></td></tr>
              <tr><td>Meter Fee</td><td>2026-04-16</td><td>$0.00</td><td></td><td></td></tr>
            </table>";
        let rates = parse(html);
        // First row wins; the duplicate does not replace it.
        assert_eq!(tou(&rates, TouPeriod::OffPeak), Some(9.8));
        assert_eq!(tou(&rates, TouPeriod::MidPeak), None);
        assert!(rates.fees.meter.is_none());
        assert!(rates.unknown.contains_key("Mid-Peak"));
        assert!(rates.unknown.contains_key("Meter Fee"));
    }

    #[test]
    fn header_rows_and_text_only_rows_are_skipped() {
        let html = "
            <table>
              <tr><th></th><th>Current</th><th>Previous</th></tr>
              <tr><th></th><th>Effective Date</th><th>Rate (&cent;/KWh)</th><th>Effective Date</th><th>Rate</th></tr>
              <tr><td>Notes</td><td>see below</td><td></td><td></td><td></td></tr>
            </table>";
        let rates = parse(html);
        assert_eq!(rates, Rates::default());
    }

    #[test]
    fn login_form_is_reported_not_parsed_as_empty() {
        let form = r#"<form action="/do-rates-login" method="POST">
            <input name="CustomerNumber"><input name="LocationID"></form>"#;
        assert_eq!(page(&Html::parse_document(form)), None);
        // No tables and no form: a changed page, not a missing session.
        assert_eq!(
            page(&Html::parse_document("<p>moved</p>")),
            Some(Rates::default())
        );
    }
}

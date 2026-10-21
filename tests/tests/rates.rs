// SPDX-License-Identifier: GPL-3.0-only

//! End-to-end tests for `client.rates` against the real corporate site.

use provident::Error;
use provident::models::pemi::rates_and_fees::parsed::{
    RateEntry, Rates, Tier, TouPeriod, UloPeriod,
};
use provident_tests::{client, credentials};

/// Signs in to the rates site with the e2e account and fetches the live page.
#[maybe_async::maybe_async]
async fn live_rates() -> Rates {
    let client = client();
    let creds = credentials();
    client
        .rates
        .login(&creds.username, &creds.password)
        .await
        .unwrap();
    client.rates.rates().await.unwrap()
}

fn current(entry: Option<&RateEntry>) -> f64 {
    entry
        .and_then(|e| e.current)
        .unwrap_or_else(|| panic!("missing current value in {entry:?}"))
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn every_published_row_is_complete() {
    let rates = live_rates().await;
    let e = &rates.electricity;
    assert_eq!(e.tiered.len(), 2, "{rates:?}");
    assert_eq!(e.tou.len(), 3, "{rates:?}");
    assert_eq!(e.ulo.len(), 4, "{rates:?}");
    let entries = e
        .tiered
        .values()
        .chain(e.tou.values())
        .chain(e.ulo.values())
        .chain(rates.fees.admin.iter())
        .chain(rates.fees.meter.iter());
    for entry in entries {
        let (Some(cur), Some(prev)) = (entry.current_effective, entry.previous_effective) else {
            panic!("missing effective dates in {entry:?}");
        };
        assert!(
            prev < cur,
            "previous rate takes effect after current: {entry:?}"
        );
        assert!(
            entry.current.is_some() && entry.previous.is_some(),
            "{entry:?}"
        );
    }
}

/// Relations the OEB's Regulated Price Plan guarantees. A parser that reads a row into the
/// wrong period (e.g. ULO's "On-Peak" as TOU's) breaks one of these even when every value is
/// individually plausible.
#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn rates_obey_rpp_invariants() {
    let rates = live_rates().await;
    let e = &rates.electricity;
    let tou = |p| current(e.tou.get(&p));
    let ulo = |p| current(e.ulo.get(&p));
    let tier = |t| current(e.tiered.get(&t));

    assert!(
        tou(TouPeriod::OffPeak) < tou(TouPeriod::MidPeak),
        "{rates:?}"
    );
    assert!(
        tou(TouPeriod::MidPeak) < tou(TouPeriod::OnPeak),
        "{rates:?}"
    );
    assert!(
        ulo(UloPeriod::UltraLowOvernight) < ulo(UloPeriod::WeekendOffPeak),
        "{rates:?}"
    );
    assert!(
        ulo(UloPeriod::WeekendOffPeak) < ulo(UloPeriod::MidPeak),
        "{rates:?}"
    );
    assert!(
        ulo(UloPeriod::MidPeak) < ulo(UloPeriod::OnPeak),
        "{rates:?}"
    );
    assert!(tier(Tier::First) < tier(Tier::Second), "{rates:?}");
    // ULO shares TOU's off- and mid-peak prices; its on-peak is the highest of any plan.
    assert_eq!(
        ulo(UloPeriod::WeekendOffPeak),
        tou(TouPeriod::OffPeak),
        "{rates:?}"
    );
    assert_eq!(
        ulo(UloPeriod::MidPeak),
        tou(TouPeriod::MidPeak),
        "{rates:?}"
    );
    assert!(ulo(UloPeriod::OnPeak) > tou(TouPeriod::OnPeak), "{rates:?}");
    // Energy rates in ¢/kWh, fees in dollars: both well inside the parser's plausibility caps.
    for value in e
        .tou
        .values()
        .chain(e.ulo.values())
        .chain(e.tiered.values())
    {
        assert!((1.0..=100.0).contains(&current(Some(value))), "{value:?}");
    }
    let admin = current(rates.fees.admin.as_ref());
    let meter = current(rates.fees.meter.as_ref());
    assert!(
        (1.0..=100.0).contains(&admin) && (1.0..=100.0).contains(&meter),
        "{rates:?}"
    );
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn portal_and_rates_sessions_coexist_on_one_client() {
    let client = client();
    let creds = credentials();
    client.auth.login(&creds).await.unwrap();
    client
        .rates
        .login(&creds.username, &creds.password)
        .await
        .unwrap();
    // Each host keeps its own cookie; neither sign-in displaces the other.
    let meters = client.account.meters().await.unwrap();
    let rates = client.rates.rates().await.unwrap();
    let portal_session = client.auth.verify().await.unwrap();
    assert!(!meters.is_empty());
    assert!(!rates.is_empty(), "{rates:?}");
    assert!(portal_session, "rates login displaced the portal session");
    // Signing in again on a live session is harmless.
    client
        .rates
        .login(&creds.username, &creds.password)
        .await
        .unwrap();
    let again = client.rates.rates().await.unwrap();
    assert!(!again.is_empty(), "{again:?}");
}

/// Pins a site quirk the `login` docs rely on: only `LocationID` is checked. If the site
/// starts validating `CustomerNumber`, this fails and the docs need updating. Sent to the rates
/// site only, never the metering portal, so it cannot lock the portal account.
#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn rates_login_only_checks_the_location() {
    let client = client();
    let creds = credentials();
    client.rates.login(&creds.username, "000000").await.unwrap();
    let rates = client.rates.rates().await.unwrap();
    assert!(!rates.is_empty(), "{rates:?}");
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn rates_match_the_published_page() {
    let rates = live_rates().await;
    let tou_on = rates
        .electricity
        .tou
        .get(&TouPeriod::OnPeak)
        .and_then(|r| r.current);
    let ulo_on = rates
        .electricity
        .ulo
        .get(&UloPeriod::OnPeak)
        .and_then(|r| r.current);
    let tier_2 = rates
        .electricity
        .tiered
        .get(&Tier::Second)
        .and_then(|r| r.current);
    let admin = rates.fees.admin.as_ref().and_then(|f| f.current);
    assert!(tou_on.is_some_and(|v| v > 0.0), "{rates:?}");
    assert!(ulo_on.is_some_and(|v| v > 0.0), "{rates:?}");
    assert!(tier_2.is_some_and(|v| v > 0.0), "{rates:?}");
    assert!(admin.is_some_and(|v| v > 0.0), "{rates:?}");
    // ULO on-peak is the plan's most expensive period; it must not be confused with TOU's.
    assert!(ulo_on.unwrap() > tou_on.unwrap(), "{rates:?}");
    // Every row on the live page is recognized; a new one means the parser needs a look.
    assert!(rates.unknown.is_empty(), "{:?}", rates.unknown);
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn rates_login_rejects_unknown_location() {
    let client = client();
    let result = client.rates.login("000-0000", "000000").await;
    assert!(
        matches!(result, Err(Error::LoginRejected { .. })),
        "{result:?}"
    );
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn rates_without_login_require_a_session() {
    // No session: the site serves its sign-in form, which must not read as "no rates".
    let result = client().rates.rates().await;
    assert!(
        matches!(result, Err(Error::RatesSessionRequired)),
        "{result:?}"
    );
}

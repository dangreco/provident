// SPDX-License-Identifier: GPL-3.0-only

//! End-to-end tests for `client.usage`.

use std::collections::BTreeSet;

use chrono::{Days, NaiveDate};
use provident::models::api::internal::graphs::quick_graphs::QuickGraphsRequest;
use provident::models::api::internal::reports::quick_report::{QuickReportRequest, RowSource};
use provident::models::api::internal::usage::MeterList;
use provident::models::common::NodeId;
use provident::{Client, Error};
use provident_tests::{client, credentials};

/// A fixed, fully elapsed range so readings are final.
fn range(days: u64) -> (NaiveDate, NaiveDate) {
    let start = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
    (start, start.checked_add_days(Days::new(days)).unwrap())
}

#[maybe_async::maybe_async]
async fn signed_in() -> Client {
    let client = client();
    client.auth.login(&credentials()).await.unwrap();
    client
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn series_resolution_follows_range_length() {
    let client = signed_in().await;
    let meter = client.account.meters().await.unwrap().remove(0);

    for (days, points) in [(1, 24), (7, 7 * 24), (8, 8), (30, 30)] {
        let (start, end) = range(days);
        let query = QuickGraphsRequest::builder()
            .meters(meter.id)
            .start(start)
            .end(end)
            .build();
        let series = client.usage.series(&query).await.unwrap();
        assert_eq!(series.len(), 1, "{days} days: {series:?}");
        assert_eq!(series[0].units, meter.unit, "{days} days");
        assert_eq!(series[0].data.len(), points, "{days} days");
    }
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn group_series_covers_every_metered_utility() {
    let client = signed_in().await;
    let meters = client.account.meters().await.unwrap();
    let group = meters[0].group.expect("meter has a group");

    let (start, end) = range(30);
    let query = QuickGraphsRequest::builder()
        .meters(group)
        .start(start)
        .end(end)
        .build();
    let series = client.usage.series(&query).await.unwrap();
    let units: BTreeSet<_> = series.iter().map(|s| s.units.as_str()).collect();
    let expected: BTreeSet<_> = meters.iter().map(|m| m.unit.as_str()).collect();
    assert_eq!(series.len(), meters.len(), "{series:?}");
    assert_eq!(units, expected);
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn report_rows_are_the_accounts_meters_and_consistent() {
    let client = signed_in().await;
    let meters = client.account.meters().await.unwrap();

    let (start, end) = range(30);
    let query = QuickReportRequest::builder()
        .meters(meters.iter().map(|m| m.id).collect::<MeterList>())
        .start(start)
        .end(end)
        .build();
    let report = client.usage.report(&query).await.unwrap();
    let rows: Vec<_> = report.values().flatten().collect();
    assert_eq!(rows.len(), meters.len(), "{report:?}");

    for row in rows {
        let RowSource::Meter(id) = row.source() else {
            panic!("expected a meter row: {row:?}");
        };
        let meter = meters
            .iter()
            .find(|m| m.id == id)
            .expect("row for a known meter");
        assert_eq!(row.serial, meter.serial);
        if let (Some(current), Some(previous), Some(total)) = (row.current, row.previous, row.total)
        {
            assert!((current - previous - total).abs() < 1e-6, "{row:?}");
        }
    }
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn aggregated_report_rows_are_the_group() {
    let client = signed_in().await;
    let meters = client.account.meters().await.unwrap();
    let group = meters[0].group.expect("meter has a group");

    let (start, end) = range(30);
    let query = QuickReportRequest::builder()
        .meters(NodeId::from(group))
        .start(start)
        .end(end)
        .aggregate_groups(true)
        .build();
    let report = client.usage.report(&query).await.unwrap();
    assert!(!report.is_empty());
    for row in report.values().flatten() {
        assert_eq!(row.source(), RowSource::Group(group), "{row:?}");
    }
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn reversed_range_is_rejected() {
    let client = signed_in().await;
    let meter = client.account.meters().await.unwrap().remove(0);
    let (start, end) = range(7);
    let query = QuickGraphsRequest::builder()
        .meters(meter.id)
        .start(end)
        .end(start)
        .build();
    let result = client.usage.series(&query).await;
    let status = match &result {
        Err(Error::Http(e)) => e.status(),
        _ => None,
    };
    assert_eq!(status.map(|s| s.as_u16()), Some(400), "{result:?}");
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn usage_is_scoped_to_the_account() {
    let client = signed_in().await;
    let meter = client.account.meters().await.unwrap().remove(0);
    // The site (building) above the account's suite contains other units' meters.
    let site = client
        .account
        .location(meter.id)
        .await
        .unwrap()
        .into_iter()
        .find(|node| node.is_site())
        .and_then(|node| node.group_id())
        .expect("meter has a site");

    let (start, end) = range(7);
    let query = QuickReportRequest::builder()
        .meters(NodeId::from(site))
        .start(start)
        .end(end)
        .include_deactivated(true)
        .build();
    let report = client.usage.report(&query).await.unwrap();
    assert!(report.is_empty(), "foreign rows visible: {report:?}");

    // Signed out, the server never answers with data (500 for real meters, empty otherwise).
    let query = QuickGraphsRequest::builder()
        .meters(meter.id)
        .start(start)
        .end(end)
        .build();
    let signed_out = provident_tests::client().usage.series(&query).await;
    assert!(
        signed_out.as_ref().map_or(true, Vec::is_empty),
        "{signed_out:?}"
    );
}

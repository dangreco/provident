// SPDX-License-Identifier: GPL-3.0-only

//! End-to-end tests for `client.account`.

use std::collections::BTreeSet;

use provident_client::Error;
use provident_tests::{client, credentials};

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn meters_are_listed_for_signed_in_account() {
    let client = client();
    client.auth.login(&credentials()).await.unwrap();

    let meters = client.account.meters().await.unwrap();
    assert!(!meters.is_empty(), "no meters returned");
    let ids: BTreeSet<_> = meters.iter().map(|m| m.id).collect();
    assert_eq!(ids.len(), meters.len(), "duplicate meter ids in {meters:?}");
    for meter in &meters {
        assert!(
            !meter.name.is_empty() && !meter.unit.is_empty() && !meter.serial.is_empty(),
            "{meter:?}"
        );
    }
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn utilities_are_those_of_the_accounts_meters() {
    let client = client();
    client.auth.login(&credentials()).await.unwrap();

    let utilities = client.account.utilities().await.unwrap();
    let unique: BTreeSet<_> = utilities.iter().copied().collect();
    assert_eq!(unique.len(), utilities.len(), "duplicates in {utilities:?}");

    let metered: BTreeSet<_> = client
        .account
        .meters()
        .await
        .unwrap()
        .iter()
        .map(|m| m.utility)
        .collect();
    assert!(!unique.is_empty(), "no utilities returned");
    assert_eq!(unique, metered);
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn account_requires_session() {
    let result = client().account.meters().await;
    let status = match &result {
        Err(Error::Http(e)) => e.status(),
        _ => None,
    };
    assert_eq!(status.map(|s| s.as_u16()), Some(401), "{result:?}");
}

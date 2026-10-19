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
async fn meters_are_found_by_id_and_serial_and_located() {
    let client = client();
    client.auth.login(&credentials()).await.unwrap();
    let meters = client.account.meters().await.unwrap();
    let first = meters.first().expect("account has meters");

    let by_id = client.account.meter(first.id).await.unwrap();
    assert_eq!(by_id.as_ref(), Some(first));
    let by_serial = client.account.meter_by_serial(&first.serial).await.unwrap();
    assert_eq!(by_serial.as_ref(), Some(first));
    let missing = client
        .account
        .meter_by_serial("no-such-serial")
        .await
        .unwrap();
    assert_eq!(missing, None);

    // Ancestors end with the meter's own group and include a site (building).
    let location = client.account.location(first.id).await.unwrap();
    assert_eq!(location.last().and_then(|n| n.group_id()), first.group);
    assert!(
        location.iter().any(|n| n.is_site()),
        "no site in {location:?}"
    );
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
async fn details_match_the_accounts_groups() {
    let client = client();
    let creds = credentials();
    client.auth.login(&creds).await.unwrap();

    let details = client.account.details().await.unwrap();
    assert_eq!(details.login, creds.username);
    assert!(!details.groups.is_empty(), "{details:?}");
    // Every meter's suite name is one of the account's groups.
    for meter in client.account.meters().await.unwrap() {
        assert!(
            details.groups.contains(&meter.name),
            "{meter:?} not in {details:?}"
        );
    }
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn api_key_requires_the_account_password() {
    let client = client();
    let creds = credentials();
    client.auth.login(&creds).await.unwrap();

    let key = client.account.api_key(&creds.password).await.unwrap();
    assert!(!key.is_empty());
    // Stable across calls (reading does not rotate it).
    let again = client.account.api_key(&creds.password).await.unwrap();
    assert_eq!(again, key);

    let rejected = client.account.api_key("not-the-password").await;
    assert!(
        matches!(&rejected, Err(Error::PasswordRejected { message: Some(m) }) if !m.is_empty()),
        "{:?}",
        rejected.map(|_| "<key>")
    );
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

// SPDX-License-Identifier: GPL-3.0-only

//! End-to-end tests for `client.account`.

use std::collections::BTreeSet;

use provident_client::Error;
use provident_tests::{client, credentials, expected_utilities};

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn utilities_match_configured_account() {
    let client = client();
    client.auth.login(&credentials()).await.unwrap();

    let utilities = client.account.utilities().await.unwrap();
    let unique: BTreeSet<_> = utilities.iter().copied().collect();
    assert_eq!(unique.len(), utilities.len(), "duplicates in {utilities:?}");
    assert_eq!(unique, expected_utilities());
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn utilities_require_session() {
    let result = client().account.utilities().await;
    let status = match &result {
        Err(Error::Http(e)) => e.status(),
        _ => None,
    };
    assert_eq!(status.map(|s| s.as_u16()), Some(401), "{result:?}");
}

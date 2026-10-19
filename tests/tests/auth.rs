// SPDX-License-Identifier: GPL-3.0-only

//! End-to-end tests for `client.auth`.

use provident::Error;
use provident::models::login::login_service::process_login::ProcessLoginRequest;
use provident_tests::{client, credentials};

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn verify_is_false_without_session() {
    let verified = client().auth.verify().await.unwrap();
    assert!(!verified);
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn login_rejects_invalid_credentials() {
    let creds = ProcessLoginRequest::builder()
        .username("provident-tests-nonexistent-user")
        .password("not-the-password")
        .build();
    let result = client().auth.login(&creds).await;
    assert!(
        matches!(&result, Err(Error::LoginRejected { message: Some(m) }) if !m.is_empty()),
        "{result:?}"
    );
}

#[ignore = "e2e"]
#[maybe_async::test(feature = "blocking", async(feature = "async", tokio::test))]
async fn login_establishes_session_shared_by_clones() {
    let client = client();
    client.auth.login(&credentials()).await.unwrap();

    let verified = client.auth.verify().await.unwrap();
    assert!(verified, "session not established after login");
    let verified_by_clone = client.clone().auth.verify().await.unwrap();
    assert!(verified_by_clone, "clone does not share the session");
}

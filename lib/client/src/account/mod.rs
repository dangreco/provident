// SPDX-License-Identifier: GPL-3.0-only

//! The signed-in account. Requires [`crate::auth::Auth::login`] first; without a session these
//! calls fail with HTTP 401. Wire endpoints: `provident_models::secure::dashboard::default`.

mod utilities;

/// Accessed as [`crate::Client::account`].
#[derive(Debug, Clone)]
pub struct Account {
    http: provident_http::Client,
}

impl Account {
    pub(crate) fn new(http: provident_http::Client) -> Self {
        Self { http }
    }
}

// SPDX-License-Identifier: GPL-3.0-only

//! Authentication. Wire endpoints: `provident_models::login::login_service`.

mod login;
mod verify;

/// Accessed as [`crate::Client::auth`].
#[derive(Debug, Clone)]
pub struct Auth {
    http: provident_http::Client,
}

impl Auth {
    pub(crate) fn new(http: provident_http::Client) -> Self {
        Self { http }
    }
}

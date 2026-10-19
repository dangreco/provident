// SPDX-License-Identifier: GPL-3.0-only

//! Authentication. Wire endpoints: `crate::models::login::login_service`.

mod login;
mod verify;

/// Accessed as [`crate::Client::auth`].
#[derive(Debug, Clone)]
pub struct Auth {
    http: crate::http::Client,
}

impl Auth {
    pub(crate) fn new(http: crate::http::Client) -> Self {
        Self { http }
    }
}

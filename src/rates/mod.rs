// SPDX-License-Identifier: GPL-3.0-only

//! Published rates and fees, from Provident's corporate site (`pemi.com`), for the location
//! the client signs in for. A separate session from the metering portal's: call
//! [`Rates::login`] (portal credentials, fields swapped) before [`Rates::rates`].
//!
//! The rates themselves are set by the OEB and local utilities; this area only reports what
//! Provident publishes for the location, as [`crate::models::pemi::rates_and_fees::parsed`].

mod login;

/// Accessed as [`crate::Client::rates`].
#[derive(Debug, Clone)]
pub struct Rates {
    pub(crate) http: crate::http::Client,
}

impl Rates {
    pub(crate) fn new(http: crate::http::Client) -> Self {
        Self { http }
    }
}

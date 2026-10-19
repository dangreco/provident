// SPDX-License-Identifier: GPL-3.0-only

//! Consumption history for meters and groups. Requires [`crate::auth::Auth::login`] first.
//! Unlike the rest of the API these endpoints do not answer 401 without a session: they return
//! an empty result, or HTTP 500 for ids that exist. Wire endpoints:
//! `crate::models::api::internal::{graphs, reports}`.
//!
//! Meters and groups are addressed by [`crate::models::common::NodeId`] (a
//! [`crate::models::api::internal::meter_tree::Meter`]'s `id`, or a group id); a group
//! covers every meter below it. Ids outside the account's own tree, such as its building's
//! site group, also yield an empty result.
//!
//! Not the dashboard's `UpdateCard`/`GetChartData`: they share the per-server state that makes
//! `GetUtilities` intermittently answer for another account.

mod report;
mod series;

/// Accessed as [`crate::Client::usage`].
#[derive(Debug, Clone)]
pub struct Usage {
    http: crate::http::Client,
}

impl Usage {
    pub(crate) fn new(http: crate::http::Client) -> Self {
        Self { http }
    }
}

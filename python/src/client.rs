// SPDX-License-Identifier: GPL-3.0-only

//! `Client` (blocking) and `AsyncClient` (asyncio), with their `auth`/`account`/`usage` areas.
//!
//! Every operation is written once, as a future over the crate's async client; [`areas!`]
//! generates the blocking method (runs it on the shared tokio runtime with the GIL released)
//! and the async method (hands it to asyncio) from it.

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::NaiveDate;
use provident::ClientConfig;
use provident::models::api::internal::graphs::quick_graphs::QuickGraphsRequest;
use provident::models::api::internal::reports::quick_report::QuickReportRequest;
use provident::models::common::MeterId;
use provident::models::login::login_service::process_login::ProcessLoginRequest;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::args::{Node, Nodes};
use crate::errors::to_py;
use crate::models::{AccountDetails, Meter, PublishedRates, ReportRow, Series, TreeNode, Utility};

/// Runs `future` to completion on the shared runtime, with the GIL released meanwhile.
fn block_on<T: Send>(
    py: Python<'_>,
    future: impl Future<Output = PyResult<T>> + Send,
) -> PyResult<T> {
    py.detach(|| pyo3_async_runtimes::tokio::get_runtime().block_on(future))
}

/// Seconds as a [`Duration`]; rejects negative, NaN, and overflowing values.
fn seconds(name: &str, value: Option<f64>) -> PyResult<Option<Duration>> {
    value
        .map(|secs| {
            Duration::try_from_secs_f64(secs).map_err(|_| {
                PyValueError::new_err(format!(
                    "{name} must be a non-negative number of seconds, got {secs}"
                ))
            })
        })
        .transpose()
}

/// The crate client for the constructor arguments shared by `Client` and `AsyncClient`.
fn build(
    base_url: Option<&str>,
    rates_base_url: Option<&str>,
    timeout: Option<f64>,
    connect_timeout: Option<f64>,
    user_agent: Option<String>,
) -> PyResult<provident::Client> {
    let config = ClientConfig::builder()
        .maybe_base_url(base_url)
        .map_err(|e| PyValueError::new_err(e.to_string()))?
        .maybe_rates_base_url(rates_base_url)
        .map_err(|e| PyValueError::new_err(e.to_string()))?
        .maybe_timeout(seconds("timeout", timeout)?)
        .maybe_connect_timeout(seconds("connect_timeout", connect_timeout)?)
        .maybe_user_agent(user_agent)
        .build();
    provident::Client::with_config(config).map_err(to_py)
}

/// Generates, per area, a blocking and an asyncio `#[pyclass]` wrapping the crate's area, with
/// one method each per operation. `$area` is a clone of the crate area, moved into `$body`
/// (an `async` block yielding `PyResult<$ret>`) together with the arguments.
macro_rules! areas {
    ($(
        $(#[$class_doc:meta])*
        $sync:ident, $async_:ident: $inner:ty {
            $(
                $(#[$method_meta:meta])*
                fn $name:ident($area:ident $(, $arg:ident: $ty:ty)*) -> $ret:ty $body:block
            )*
        }
    )*) => {$(
        $(#[$class_doc])*
        #[pyclass(frozen, module = "provident")]
        pub struct $sync($inner);

        $(#[$class_doc])*
        #[pyclass(frozen, module = "provident")]
        pub struct $async_($inner);

        #[pymethods]
        impl $sync {
            $(
                $(#[$method_meta])*
                fn $name(&self, py: Python<'_> $(, $arg: $ty)*) -> PyResult<$ret> {
                    let $area = self.0.clone();
                    block_on(py, async move $body)
                }
            )*
        }

        #[pymethods]
        impl $async_ {
            $(
                $(#[$method_meta])*
                fn $name<'py>(&self, py: Python<'py> $(, $arg: $ty)*) -> PyResult<Bound<'py, PyAny>> {
                    let $area = self.0.clone();
                    pyo3_async_runtimes::tokio::future_into_py::<_, $ret>(py, async move $body)
                }
            )*
        }
    )*};
}

areas! {
    /// Sign-in and session check; accessed as `client.auth`.
    Auth, AsyncAuth: provident::auth::Auth {
        /// Signs in; the session is shared by every later call on this client.
        /// Raises `LoginRejectedError` when the server refuses the credentials.
        #[pyo3(signature = (username, password, *, remember_me = false))]
        fn login(auth, username: String, password: String, remember_me: bool) -> () {
            let request = ProcessLoginRequest::builder()
                .username(username)
                .password(password)
                .remember_me(remember_me)
                .build();
            auth.login(&request).await.map_err(to_py)
        }

        /// Whether the client currently holds a valid session.
        fn verify(auth) -> bool {
            auth.verify().await.map_err(to_py)
        }
    }

    /// The signed-in account; accessed as `client.account`. Requires `auth.login` first.
    Account, AsyncAccount: provident::account::Account {
        /// Every metering point visible to the account, in tree order.
        fn meters(account) -> Vec<Meter> {
            let meters = account.meters().await.map_err(to_py)?;
            Ok(meters.into_iter().map(Meter).collect())
        }

        /// The account's meter with this id, if any.
        fn meter(account, id: u64) -> Option<Meter> {
            Ok(account.meter(MeterId(id)).await.map_err(to_py)?.map(Meter))
        }

        /// The account's meter with this serial number ("Meter #"), if any.
        fn meter_by_serial(account, serial: String) -> Option<Meter> {
            Ok(account.meter_by_serial(&serial).await.map_err(to_py)?.map(Meter))
        }

        /// Distinct utilities of the account's meters, in the order first seen.
        fn utilities(account) -> Vec<Utility> {
            let utilities = account.utilities().await.map_err(to_py)?;
            Ok(utilities.into_iter().map(Utility).collect())
        }

        /// Groups above `node` (a `Meter`, `TreeNode`, or node id), root first.
        fn location(account, node: Node) -> Vec<TreeNode> {
            let nodes = account.location(node.0).await.map_err(to_py)?;
            Ok(nodes.into_iter().map(TreeNode).collect())
        }

        /// The signed-in user's profile.
        fn details(account) -> AccountDetails {
            Ok(AccountDetails(account.details().await.map_err(to_py)?))
        }

        /// The account's API key; the portal requires the account password again.
        /// Raises `PasswordRejectedError` for a wrong password, which also counts as a failed
        /// sign-in on the account.
        fn api_key(account, password: String) -> String {
            account.api_key(&password).await.map_err(to_py)
        }
    }

    /// Consumption history; accessed as `client.usage`. Requires `auth.login` first.
    Usage, AsyncUsage: provident::usage::Usage {
        /// Consumption series, one per utility: hourly for ranges up to 7 days, daily beyond.
        /// `end` is exclusive.
        #[pyo3(signature = (meters, start, end, *, aggregate_groups = false))]
        fn series(usage, meters: Nodes, start: NaiveDate, end: NaiveDate, aggregate_groups: bool) -> Vec<Series> {
            let query = QuickGraphsRequest::builder()
                .meters(meters.0)
                .start(start)
                .end(end)
                .aggregate_groups(aggregate_groups)
                .build();
            let series = usage.series(&query).await.map_err(to_py)?;
            Ok(series.into_iter().map(Series).collect())
        }

        /// Start/end readings and totals per meter, keyed by utility label. `end` is exclusive.
        #[pyo3(signature = (meters, start, end, *, aggregate_groups = false, include_deactivated = false))]
        fn report(
            usage,
            meters: Nodes,
            start: NaiveDate,
            end: NaiveDate,
            aggregate_groups: bool,
            include_deactivated: bool
        ) -> BTreeMap<String, Vec<ReportRow>> {
            let query = QuickReportRequest::builder()
                .meters(meters.0)
                .start(start)
                .end(end)
                .aggregate_groups(aggregate_groups)
                .include_deactivated(include_deactivated)
                .build();
            let report = usage.report(&query).await.map_err(to_py)?;
            Ok(report
                .into_iter()
                .map(|(utility, rows)| (utility, rows.into_iter().map(ReportRow).collect()))
                .collect())
        }
    }

    /// Published rates and fees; accessed as `client.rates`. Requires `rates.login` first.
    Rates, AsyncRates: provident::rates::Rates {
        /// Signs in to the rates site with the metering portal's credentials (username is the
        /// location id, password the customer number). Raises `LoginRejectedError` when the
        /// location id is refused.
        #[pyo3(signature = (username, password))]
        fn login(area, username: String, password: String) -> () {
            area.login(&username, &password).await.map_err(to_py)
        }

        /// The location's published rates and fees; requires `rates.login` first. Layout
        /// changes on the page degrade to `None` fields, never an error.
        fn fetch(area) -> PublishedRates {
            Ok(PublishedRates(area.rates().await.map_err(to_py)?))
        }
    }
}

/// Blocking client. Calls release the GIL while waiting on the network.
#[pyclass(frozen, module = "provident")]
pub struct Client {
    /// Sign-in and session check.
    #[pyo3(get)]
    auth: Py<Auth>,
    /// The signed-in account's meters and profile.
    #[pyo3(get)]
    account: Py<Account>,
    /// Consumption history.
    #[pyo3(get)]
    usage: Py<Usage>,
    /// Published rates and fees (`pemi.com`).
    #[pyo3(get)]
    rates: Py<Rates>,
}

#[pymethods]
impl Client {
    #[new]
    #[pyo3(signature = (*, base_url = None, rates_base_url = None, timeout = None, connect_timeout = None, user_agent = None))]
    fn new(
        py: Python<'_>,
        base_url: Option<&str>,
        rates_base_url: Option<&str>,
        timeout: Option<f64>,
        connect_timeout: Option<f64>,
        user_agent: Option<String>,
    ) -> PyResult<Self> {
        let client = build(
            base_url,
            rates_base_url,
            timeout,
            connect_timeout,
            user_agent,
        )?;
        Ok(Self {
            auth: Py::new(py, Auth(client.auth))?,
            account: Py::new(py, Account(client.account))?,
            usage: Py::new(py, Usage(client.usage))?,
            rates: Py::new(py, Rates(client.rates))?,
        })
    }
}

/// asyncio client: every call returns an awaitable.
#[pyclass(frozen, module = "provident")]
pub struct AsyncClient {
    /// Sign-in and session check.
    #[pyo3(get)]
    auth: Py<AsyncAuth>,
    /// The signed-in account's meters and profile.
    #[pyo3(get)]
    account: Py<AsyncAccount>,
    /// Consumption history.
    #[pyo3(get)]
    usage: Py<AsyncUsage>,
    /// Published rates and fees (`pemi.com`).
    #[pyo3(get)]
    rates: Py<AsyncRates>,
}

#[pymethods]
impl AsyncClient {
    #[new]
    #[pyo3(signature = (*, base_url = None, rates_base_url = None, timeout = None, connect_timeout = None, user_agent = None))]
    fn new(
        py: Python<'_>,
        base_url: Option<&str>,
        rates_base_url: Option<&str>,
        timeout: Option<f64>,
        connect_timeout: Option<f64>,
        user_agent: Option<String>,
    ) -> PyResult<Self> {
        let client = build(
            base_url,
            rates_base_url,
            timeout,
            connect_timeout,
            user_agent,
        )?;
        Ok(Self {
            auth: Py::new(py, AsyncAuth(client.auth))?,
            account: Py::new(py, AsyncAccount(client.account))?,
            usage: Py::new(py, AsyncUsage(client.usage))?,
            rates: Py::new(py, AsyncRates(client.rates))?,
        })
    }
}

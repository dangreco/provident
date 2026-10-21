// SPDX-License-Identifier: GPL-3.0-only

//! Python bindings for [`provident`], built with maturin as `provident._provident` and
//! re-exported by the `provident` Python package (`python/provident/__init__.py`).
//!
//! Both Python clients drive the crate's async client on one shared tokio runtime:
//! `AsyncClient` hands each call to asyncio as a future, `Client` blocks on it with the GIL
//! released. `Utility` and the exceptions are defined in Python (`provident/_types.py`) and
//! converted or raised from here.

use pyo3::prelude::*;

mod args;
mod client;
mod errors;
mod models;

#[pymodule]
fn _provident(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    // Canonical utility names; the Python `Utility` enum must cover every one.
    m.add("_UTILITIES", models::utility_names())?;
    m.add_class::<models::Meter>()?;
    m.add_class::<models::TreeNode>()?;
    m.add_class::<models::Series>()?;
    m.add_class::<models::ReportRow>()?;
    m.add_class::<models::AccountDetails>()?;
    m.add_class::<models::RateEntry>()?;
    m.add_class::<models::PublishedRates>()?;
    m.add_class::<client::Client>()?;
    m.add_class::<client::AsyncClient>()?;
    m.add_class::<client::Auth>()?;
    m.add_class::<client::Account>()?;
    m.add_class::<client::Usage>()?;
    m.add_class::<client::AsyncAuth>()?;
    m.add_class::<client::AsyncAccount>()?;
    m.add_class::<client::AsyncUsage>()?;
    m.add_class::<client::Rates>()?;
    m.add_class::<client::AsyncRates>()?;
    Ok(())
}

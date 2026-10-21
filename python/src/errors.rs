// SPDX-License-Identifier: GPL-3.0-only

//! [`provident::Error`] → the Python exceptions in `provident/_types.py`.

use provident::Error;
use provident::http::Error as HttpError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Converts to the matching `provident` exception, e.g. `.map_err(errors::to_py)?`.
pub fn to_py(error: Error) -> PyErr {
    Python::attach(|py| to_pyerr(py, &error).unwrap_or_else(|e| e))
}

/// Builds `provident.<class>(str(error), **kwargs)`; `Err` carries a failure to import or
/// construct it.
fn to_pyerr(py: Python<'_>, error: &Error) -> PyResult<PyErr> {
    let kwargs = PyDict::new(py);
    let class = match error {
        Error::LoginRejected { message } => {
            kwargs.set_item("reason", message)?;
            "LoginRejectedError"
        }
        Error::PasswordRejected { message } => {
            kwargs.set_item("reason", message)?;
            "PasswordRejectedError"
        }
        Error::InvalidMeter(_) => "InvalidMeterError",
        Error::RatesSessionRequired => "RatesSessionRequiredError",
        Error::Http(http) => match http {
            HttpError::Status { status, body } => {
                kwargs.set_item("status", status.as_u16())?;
                kwargs.set_item("body", body)?;
                "HTTPStatusError"
            }
            HttpError::Deserialize { status, body, .. } => {
                kwargs.set_item("status", status.as_u16())?;
                kwargs.set_item("body", body)?;
                "DecodeError"
            }
            HttpError::Transport(e) => {
                kwargs.set_item("status", e.status().map(|s| s.as_u16()))?;
                kwargs.set_item("timeout", e.is_timeout())?;
                "TransportError"
            }
            // Invalid URLs/queries/bodies: client-side bugs or bad configuration.
            _ => "ProvidentError",
        },
    };
    let instance = py
        .import("provident._types")?
        .getattr(class)?
        .call((error_chain(error),), Some(&kwargs))?;
    Ok(PyErr::from_value(instance))
}

/// `error: cause: cause…`, since `Display` of transport errors omits the underlying reason.
fn error_chain(error: &Error) -> String {
    let mut message = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        let cause_text = cause.to_string();
        if !message.ends_with(&cause_text) {
            message.push_str(": ");
            message.push_str(&cause_text);
        }
        source = cause.source();
    }
    message
}

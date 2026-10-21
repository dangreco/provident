# SPDX-License-Identifier: GPL-3.0-only

"""Unofficial client for the Provident (MeterConnex) utility metering portal.

Python bindings for the `provident` Rust crate. `Client` blocks; `AsyncClient` is for asyncio.
"""

from provident._provident import (
    Account,
    AccountDetails,
    AsyncAccount,
    AsyncAuth,
    AsyncClient,
    AsyncRates,
    AsyncUsage,
    Auth,
    Client,
    Meter,
    PublishedRates,
    RateEntry,
    Rates,
    ReportRow,
    Series,
    TreeNode,
    Usage,
    __version__,
)
from provident._types import (
    DecodeError,
    HTTPStatusError,
    InvalidMeterError,
    LoginRejectedError,
    PasswordRejectedError,
    ProvidentError,
    RatesSessionRequiredError,
    TransportError,
    Utility,
)

__all__ = [
    "Account",
    "AccountDetails",
    "AsyncAccount",
    "AsyncAuth",
    "AsyncClient",
    "AsyncRates",
    "AsyncUsage",
    "Auth",
    "Client",
    "DecodeError",
    "HTTPStatusError",
    "InvalidMeterError",
    "LoginRejectedError",
    "Meter",
    "PasswordRejectedError",
    "ProvidentError",
    "PublishedRates",
    "RateEntry",
    "Rates",
    "RatesSessionRequiredError",
    "ReportRow",
    "Series",
    "TransportError",
    "TreeNode",
    "Usage",
    "Utility",
    "__version__",
]

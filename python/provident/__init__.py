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
    AsyncUsage,
    Auth,
    Client,
    Meter,
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
    TransportError,
    Utility,
)

__all__ = [
    "Account",
    "AccountDetails",
    "AsyncAccount",
    "AsyncAuth",
    "AsyncClient",
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
    "ReportRow",
    "Series",
    "TransportError",
    "TreeNode",
    "Usage",
    "Utility",
    "__version__",
]

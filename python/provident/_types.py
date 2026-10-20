# SPDX-License-Identifier: GPL-3.0-only

"""Python-side types the extension module converts to and raises."""

from __future__ import annotations

from enum import StrEnum


class Utility(StrEnum):
    """A metered utility. Values are the portal's canonical names."""

    ELECTRICITY = "Electricity"
    COLD_WATER = "Cold Water"
    HOT_WATER = "Hot Water"
    HEATING = "Heating"
    COOLING = "Cooling"
    GAS = "Gas"
    ELECTRIC_VEHICLE = "EV"


class ProvidentError(Exception):
    """Base class of every error raised by this package."""


class LoginRejectedError(ProvidentError):
    """The server answered but refused the credentials."""

    def __init__(self, message: str, *, reason: str | None) -> None:
        super().__init__(message)
        #: Reason reported by the server, if any.
        self.reason = reason


class PasswordRejectedError(ProvidentError):
    """The server answered but refused the account password (e.g. for the API key)."""

    def __init__(self, message: str, *, reason: str | None) -> None:
        super().__init__(message)
        #: Reason reported by the server, if any.
        self.reason = reason


class InvalidMeterError(ProvidentError):
    """A meter in the account's meter tree could not be read."""


class HTTPStatusError(ProvidentError):
    """The server answered with a non-2xx status (e.g. 401 without a session)."""

    def __init__(self, message: str, *, status: int, body: str) -> None:
        super().__init__(message)
        #: HTTP status code.
        self.status = status
        #: Leading part of the response body.
        self.body = body


class DecodeError(ProvidentError):
    """The response body was not in the expected shape."""

    def __init__(self, message: str, *, status: int, body: str) -> None:
        super().__init__(message)
        #: HTTP status code of the response.
        self.status = status
        #: Leading part of the response body.
        self.body = body


class TransportError(ProvidentError):
    """Connection, TLS, timeout, or other network failure."""

    def __init__(self, message: str, *, status: int | None, timeout: bool) -> None:
        super().__init__(message)
        #: HTTP status, if a response was received.
        self.status = status
        #: Whether the request timed out.
        self.timeout = timeout

# SPDX-License-Identifier: GPL-3.0-only

"""Type stubs for the compiled extension module."""

from collections.abc import Awaitable, Iterable
from datetime import date
from typing import Self, TypeAlias, final

from provident._types import Utility

__all__ = [
    "_UTILITIES",
    "Account",
    "AccountDetails",
    "AsyncAccount",
    "AsyncAuth",
    "AsyncClient",
    "AsyncRates",
    "AsyncUsage",
    "Auth",
    "Client",
    "Meter",
    "PublishedRates",
    "RateEntry",
    "Rates",
    "ReportRow",
    "Series",
    "TreeNode",
    "Usage",
    "__version__",
]

__version__: str
#: Canonical utility names; `Utility` must have exactly these values.
_UTILITIES: list[str]

_Node: TypeAlias = Meter | TreeNode | str
"""A meter-tree node: a `Meter`, a `TreeNode`, or a node id (`"MP:<n>"`, `"GROUP:<n>"`)."""

@final
class Meter:
    """A metering point on the account."""

    @property
    def id(self) -> int: ...
    @property
    def node_id(self) -> str: ...
    @property
    def group(self) -> int | None: ...
    @property
    def group_node_id(self) -> str | None: ...
    @property
    def name(self) -> str: ...
    @property
    def utility(self) -> Utility: ...
    @property
    def unit(self) -> str: ...
    @property
    def serial(self) -> str: ...

@final
class TreeNode:
    """A node of the meter tree: a group, site (building), or metering point."""

    @property
    def id(self) -> str: ...
    @property
    def parent(self) -> str | None: ...
    @property
    def text(self) -> str: ...
    @property
    def icon(self) -> str: ...
    @property
    def kind(self) -> str | None: ...
    @property
    def title(self) -> str | None: ...
    @property
    def group_id(self) -> int | None: ...
    @property
    def meter_id(self) -> int | None: ...
    @property
    def is_site(self) -> bool: ...
    @property
    def has_unloaded_children(self) -> bool: ...

@final
class Series:
    """Consumption series of one utility."""

    @property
    def utility(self) -> str: ...
    @property
    def units(self) -> str: ...
    @property
    def name(self) -> str: ...
    @property
    def site(self) -> str: ...
    @property
    def data(self) -> list[float]: ...

@final
class ReportRow:
    """One row of a usage report."""

    @property
    def name(self) -> str: ...
    @property
    def serial(self) -> str: ...
    @property
    def current(self) -> float | None: ...
    @property
    def previous(self) -> float | None: ...
    @property
    def total(self) -> float | None: ...
    @property
    def daily_average(self) -> float | None: ...
    @property
    def error(self) -> bool: ...
    @property
    def meter_id(self) -> int | None: ...
    @property
    def group_id(self) -> int | None: ...

@final
class AccountDetails:
    """Profile of the signed-in account."""

    @property
    def login(self) -> str: ...
    @property
    def first_name(self) -> str: ...
    @property
    def last_name(self) -> str: ...
    @property
    def email(self) -> str | None: ...
    @property
    def groups(self) -> list[str]: ...

@final
class RateEntry:
    """One current/previous rate pair from the published rates page."""

    @property
    def current_effective(self) -> date | None: ...
    @property
    def current(self) -> float | None: ...
    @property
    def previous_effective(self) -> date | None: ...
    @property
    def previous(self) -> float | None: ...
    def value(self) -> float | None: ...

@final
class PublishedRates:
    """Rates and fees Provident publishes for the account's location."""

    @property
    def tier_1(self) -> RateEntry | None: ...
    @property
    def tier_2(self) -> RateEntry | None: ...
    @property
    def tou_off_peak(self) -> RateEntry | None: ...
    @property
    def tou_mid_peak(self) -> RateEntry | None: ...
    @property
    def tou_on_peak(self) -> RateEntry | None: ...
    @property
    def ulo_ultra_low_overnight(self) -> RateEntry | None: ...
    @property
    def ulo_weekend_off_peak(self) -> RateEntry | None: ...
    @property
    def ulo_mid_peak(self) -> RateEntry | None: ...
    @property
    def ulo_on_peak(self) -> RateEntry | None: ...
    @property
    def admin_fee(self) -> RateEntry | None: ...
    @property
    def meter_fee(self) -> RateEntry | None: ...
    @property
    def sections(self) -> dict[str, dict[str, RateEntry]]: ...
    @property
    def unknown(self) -> dict[str, list[str]]: ...

@final
class Rates:
    def login(self, username: str, password: str) -> None: ...
    def fetch(self) -> PublishedRates: ...

@final
class AsyncRates:
    def login(self, username: str, password: str) -> Awaitable[None]: ...
    def fetch(self) -> Awaitable[PublishedRates]: ...

@final
class Auth:
    def login(self, username: str, password: str, *, remember_me: bool = False) -> None: ...
    def verify(self) -> bool: ...

@final
class Account:
    def meters(self) -> list[Meter]: ...
    def meter(self, id: int) -> Meter | None: ...
    def meter_by_serial(self, serial: str) -> Meter | None: ...
    def utilities(self) -> list[Utility]: ...
    def location(self, node: _Node) -> list[TreeNode]: ...
    def details(self) -> AccountDetails: ...
    def api_key(self, password: str) -> str: ...

@final
class Usage:
    def series(
        self,
        meters: _Node | Iterable[_Node],
        start: date,
        end: date,
        *,
        aggregate_groups: bool = False,
    ) -> list[Series]: ...
    def report(
        self,
        meters: _Node | Iterable[_Node],
        start: date,
        end: date,
        *,
        aggregate_groups: bool = False,
        include_deactivated: bool = False,
    ) -> dict[str, list[ReportRow]]: ...

@final
class AsyncAuth:
    def login(
        self, username: str, password: str, *, remember_me: bool = False
    ) -> Awaitable[None]: ...
    def verify(self) -> Awaitable[bool]: ...

@final
class AsyncAccount:
    def meters(self) -> Awaitable[list[Meter]]: ...
    def meter(self, id: int) -> Awaitable[Meter | None]: ...
    def meter_by_serial(self, serial: str) -> Awaitable[Meter | None]: ...
    def utilities(self) -> Awaitable[list[Utility]]: ...
    def location(self, node: _Node) -> Awaitable[list[TreeNode]]: ...
    def details(self) -> Awaitable[AccountDetails]: ...
    def api_key(self, password: str) -> Awaitable[str]: ...

@final
class AsyncUsage:
    def series(
        self,
        meters: _Node | Iterable[_Node],
        start: date,
        end: date,
        *,
        aggregate_groups: bool = False,
    ) -> Awaitable[list[Series]]: ...
    def report(
        self,
        meters: _Node | Iterable[_Node],
        start: date,
        end: date,
        *,
        aggregate_groups: bool = False,
        include_deactivated: bool = False,
    ) -> Awaitable[dict[str, list[ReportRow]]]: ...

@final
class Client:
    """Blocking client. Calls release the GIL while waiting on the network."""

    def __new__(
        cls,
        *,
        base_url: str | None = None,
        rates_base_url: str | None = None,
        timeout: float | None = None,
        connect_timeout: float | None = None,
        user_agent: str | None = None,
    ) -> Self: ...
    @property
    def auth(self) -> Auth: ...
    @property
    def account(self) -> Account: ...
    @property
    def usage(self) -> Usage: ...
    @property
    def rates(self) -> Rates: ...

@final
class AsyncClient:
    """asyncio client: every call returns an awaitable."""

    def __new__(
        cls,
        *,
        base_url: str | None = None,
        rates_base_url: str | None = None,
        timeout: float | None = None,
        connect_timeout: float | None = None,
        user_agent: str | None = None,
    ) -> Self: ...
    @property
    def auth(self) -> AsyncAuth: ...
    @property
    def account(self) -> AsyncAccount: ...
    @property
    def usage(self) -> AsyncUsage: ...
    @property
    def rates(self) -> AsyncRates: ...

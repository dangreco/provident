# SPDX-License-Identifier: GPL-3.0-only

"""End-to-end tests against the real portal (`pytest -m e2e`). The Rust e2e suite covers the
endpoints in depth; these check the bindings end to end through both clients.

Like the Rust suite, never send a wrong account password: failed sign-ins lock the account.
"""

from __future__ import annotations

import asyncio
from datetime import date, timedelta

import pytest

from provident import (
    AsyncClient,
    Client,
    HTTPStatusError,
    LoginRejectedError,
    PublishedRates,
    RateEntry,
    RatesSessionRequiredError,
)

pytestmark = pytest.mark.e2e

START = date(2026, 9, 1)


def test_sign_in_and_read_everything(credentials: tuple[str, str], base_url: str | None) -> None:
    username, password = credentials
    client = Client(base_url=base_url)
    assert client.auth.verify() is False
    client.auth.login(username, password)
    assert client.auth.verify() is True

    meters = client.account.meters()
    assert meters, "no meters returned"
    first = meters[0]
    assert client.account.meter(first.id) == first
    assert client.account.meter_by_serial(first.serial) == first
    assert set(client.account.utilities()) == {m.utility for m in meters}

    location = client.account.location(first)
    assert location[-1].group_id == first.group
    assert any(node.is_site for node in location)

    details = client.account.details()
    assert details.login == username
    assert first.name in details.groups

    series = client.usage.series(first, START, START + timedelta(days=1))
    assert [(s.units, len(s.data)) for s in series] == [(first.unit, 24)]

    report = client.usage.report(meters, START, START + timedelta(days=30))
    rows = [row for rows in report.values() for row in rows]
    assert sorted(row.meter_id or 0 for row in rows) == sorted(m.id for m in meters)


def test_async_client_matches_sync(credentials: tuple[str, str], base_url: str | None) -> None:
    username, password = credentials

    async def main() -> None:
        client = AsyncClient(base_url=base_url)
        await client.auth.login(username, password)
        meters, details = await asyncio.gather(client.account.meters(), client.account.details())
        assert meters and details.login == username
        group = meters[0].group_node_id
        assert group is not None
        report = await client.usage.report(
            group, START, START + timedelta(days=30), aggregate_groups=True
        )
        assert report
        assert {row.group_id for rows in report.values() for row in rows} == {meters[0].group}

    asyncio.run(main())


def test_rejections_map_to_exceptions(base_url: str | None) -> None:
    client = Client(base_url=base_url)
    with pytest.raises(LoginRejectedError) as info:
        client.auth.login("provident-tests-nonexistent-user", "not-the-password")
    assert info.value.reason

    with pytest.raises(HTTPStatusError) as status:
        client.account.meters()
    assert status.value.status == 401


def _entries(rates: PublishedRates) -> list[RateEntry]:
    entries = [
        rates.tier_1,
        rates.tier_2,
        rates.tou_off_peak,
        rates.tou_mid_peak,
        rates.tou_on_peak,
        rates.ulo_ultra_low_overnight,
        rates.ulo_weekend_off_peak,
        rates.ulo_mid_peak,
        rates.ulo_on_peak,
        rates.admin_fee,
        rates.meter_fee,
    ]
    assert all(e is not None for e in entries), rates
    return [e for e in entries if e is not None]


def _current(entry: RateEntry | None) -> float:
    assert entry is not None and entry.current is not None, entry
    return entry.current


def test_published_rates(credentials: tuple[str, str]) -> None:
    username, password = credentials
    client = Client()
    with pytest.raises(RatesSessionRequiredError):
        client.rates.fetch()
    client.rates.login(username, password)
    rates = client.rates.fetch()

    for entry in _entries(rates):
        assert entry.current_effective and entry.previous_effective, entry
        assert entry.previous_effective < entry.current_effective, entry
        assert entry.current and entry.previous, entry
    assert rates.unknown == {}, rates.unknown
    # RPP ordering: a period read into the wrong slot breaks one of these.
    assert _current(rates.tou_off_peak) < _current(rates.tou_mid_peak) < _current(rates.tou_on_peak)
    assert (
        _current(rates.ulo_ultra_low_overnight)
        < _current(rates.ulo_weekend_off_peak)
        < _current(rates.ulo_mid_peak)
        < _current(rates.ulo_on_peak)
    )
    assert _current(rates.tier_1) < _current(rates.tier_2)
    assert _current(rates.ulo_on_peak) > _current(rates.tou_on_peak)


def test_async_rates_share_a_client_with_the_portal(credentials: tuple[str, str]) -> None:
    username, password = credentials

    async def main() -> None:
        client = AsyncClient()
        await client.auth.login(username, password)
        await client.rates.login(username, password)
        meters, rates = await asyncio.gather(client.account.meters(), client.rates.fetch())
        assert meters
        assert rates.tou_on_peak is not None and rates.unknown == {}
        assert await client.auth.verify()

    asyncio.run(main())


def test_rates_login_rejects_unknown_location() -> None:
    with pytest.raises(LoginRejectedError) as info:
        Client().rates.login("000-0000", "000000")
    assert info.value.reason

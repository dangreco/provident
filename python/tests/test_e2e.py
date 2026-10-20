# SPDX-License-Identifier: GPL-3.0-only

"""End-to-end tests against the real portal (`pytest -m e2e`). The Rust e2e suite covers the
endpoints in depth; these check the bindings end to end through both clients.

Like the Rust suite, never send a wrong account password: failed sign-ins lock the account.
"""

from __future__ import annotations

import asyncio
from datetime import date, timedelta

import pytest

from provident import AsyncClient, Client, HTTPStatusError, LoginRejectedError

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

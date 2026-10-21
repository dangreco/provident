# SPDX-License-Identifier: GPL-3.0-only

"""Offline tests of the bindings against a local fake portal: argument conversion, result
objects, and error mapping, for both clients."""

from __future__ import annotations

import asyncio
import json
import time
from concurrent.futures import ThreadPoolExecutor
from datetime import date
from pathlib import Path
from typing import Any

import pytest

import provident
from provident import (
    AsyncClient,
    Client,
    DecodeError,
    HTTPStatusError,
    LoginRejectedError,
    PasswordRejectedError,
    RatesSessionRequiredError,
    TransportError,
    Utility,
)

from .conftest import (
    RATES_HTML,
    FakePortal,
    page_method,
    rates_login_ok,
    rates_login_rejected,
)

TREE = "/api/internal/metertree/rootnodes"
CHILDREN = "/api/internal/metertree/getchildren"
GRAPHS = "/api/internal/graphs/quickgraphs"
REPORT = "/api/internal/reports/quickreport"
LOGIN = "/login/LoginService.aspx/ProcessLogin"
GET_KEY = "/secure/AccountInfo.aspx/GetKey"

ROOT_NODES = [
    {"id": "GROUP:7", "text": "101", "parent": "#", "icon": "fas fa-folder"},
    {
        "id": "MP:11",
        "text": "101",
        "parent": "GROUP:7",
        "icon": "x",
        "a_attr": {"title": "101 [DCW - m3] - 1111-1"},
    },
    {"id": "GROUP:8", "text": "Storage", "parent": "GROUP:7", "icon": "x", "children": True},
]
CHILD_NODES = [
    {
        "id": "MP:12",
        "text": "101",
        "parent": "GROUP:8",
        "icon": "x",
        "a_attr": {"title": "101 [Electricity - kWh] - 2222-2"},
    },
]


def run(client: Client | AsyncClient, call: Any) -> Any:
    """`call(client)`, awaited on a fresh event loop for the async client, so tests cover both.
    Async calls need a running loop, so the call itself happens inside the coroutine."""
    if isinstance(client, AsyncClient):
        return asyncio.run(_call(client, call))
    return call(client)


async def _call(client: AsyncClient, call: Any) -> Any:
    return await call(client)


@pytest.fixture(params=[Client, AsyncClient], ids=["sync", "async"])
def client(request: pytest.FixtureRequest, portal: FakePortal) -> Client | AsyncClient:
    factory: type[Client | AsyncClient] = request.param
    return factory(base_url=portal.url, rates_base_url=portal.url)


def test_utility_enum_covers_every_crate_utility() -> None:
    assert [u.value for u in Utility] == provident._provident._UTILITIES


def test_meters_walk_the_tree_and_parse_titles(
    client: Client | AsyncClient, portal: FakePortal
) -> None:
    portal.routes[TREE] = (200, ROOT_NODES)
    portal.routes[CHILDREN] = (200, CHILD_NODES)

    meters = run(client, lambda c: c.account.meters())

    assert [(m.id, m.node_id, m.utility, m.unit, m.serial) for m in meters] == [
        (11, "MP:11", Utility.COLD_WATER, "m3", "1111-1"),
        (12, "MP:12", Utility.ELECTRICITY, "kWh", "2222-2"),
    ]
    assert (meters[1].group, meters[1].group_node_id) == (8, "GROUP:8")
    assert meters[0].utility is Utility.COLD_WATER
    # The unloaded group was fetched by its bare id.
    assert [r[2] for r in portal.requests if r[1] == CHILDREN] == [{"groupId": ["8"]}]
    assert run(client, lambda c: c.account.utilities()) == [Utility.COLD_WATER, Utility.ELECTRICITY]


def test_series_accepts_meters_nodes_and_ids(
    client: Client | AsyncClient, portal: FakePortal
) -> None:
    portal.routes[TREE] = (200, ROOT_NODES[:2])
    portal.routes[GRAPHS] = (
        200,
        [
            {
                "utility": "Cold Water (m3)",
                "units": "m3",
                "name": "101|101|T",
                "site": "T",
                "data": [0.5, 0.0],
            }
        ],
    )
    meter = run(client, lambda c: c.account.meters())[0]

    for meters in (meter, "GROUP:7", [meter, "GROUP:7"], (n for n in ["MP:1"])):
        series = run(
            client,
            lambda c, meters=meters: c.usage.series(
                meters, date(2026, 9, 1), date(2026, 9, 2), aggregate_groups=True
            ),
        )
        assert [(s.utility, s.units, s.site, s.data) for s in series] == [
            ("Cold Water (m3)", "m3", "T", [0.5, 0.0])
        ]

    queries = [r[2] for r in portal.requests if r[1] == GRAPHS]
    assert [q["meterlist"] for q in queries] == [
        ["MP:11"],
        ["GROUP:7"],
        ["MP:11,GROUP:7"],
        ["MP:1"],
    ]
    assert queries[0]["startDate"] == ["2026-09-01"]
    assert queries[0]["endDate"] == ["2026-09-02"]
    assert queries[0]["aggregateGroups"] == ["true"]


def test_report_rows_distinguish_meters_and_groups(
    client: Client | AsyncClient, portal: FakePortal
) -> None:
    row = {
        "Name": "101",
        "Current": 2.5,
        "Previous": 1.0,
        "Total": 1.5,
        "Daily Average": 0.05,
        "Error": False,
    }
    portal.routes[REPORT] = (
        200,
        {
            "Cold Water (m3)": [{**row, "Meter #": "1111-1", "ID": 11}],
            "Hot Water (m3)": [
                {**row, "Meter #": "", "ID": -7, "Current": None, "Total": None, "Error": True}
            ],
        },
    )

    report = run(client, lambda c: c.usage.report("GROUP:7", date(2026, 9, 1), date(2026, 10, 1)))

    cold, hot = report["Cold Water (m3)"][0], report["Hot Water (m3)"][0]
    assert (cold.meter_id, cold.group_id, cold.total, cold.error) == (11, None, 1.5, False)
    assert (hot.meter_id, hot.group_id, hot.current, hot.total, hot.error) == (
        None,
        7,
        None,
        None,
        True,
    )
    query = next(r[2] for r in portal.requests if r[1] == REPORT)
    assert query["includeDeactivated"] == ["false"]


@pytest.mark.parametrize(
    ("meters", "error"),
    [("692294", ValueError), ("MP:x", ValueError), ([], None), (42, TypeError), ([1], TypeError)],
)
def test_invalid_node_arguments_are_rejected(
    client: Client | AsyncClient, portal: FakePortal, meters: object, error: type[Exception] | None
) -> None:
    portal.routes[GRAPHS] = (400, {"Message": "empty meterlist"})
    # An empty list is valid Python-side; the server rejects it.
    expected = error or HTTPStatusError
    with pytest.raises(expected):
        run(client, lambda c: c.usage.series(meters, date(2026, 9, 1), date(2026, 9, 2)))
    if error is not None:
        assert not portal.requests


def test_login_sends_credentials_and_maps_rejection(
    client: Client | AsyncClient, portal: FakePortal
) -> None:
    portal.routes[LOGIN] = (200, page_method({"success": True}))
    run(client, lambda c: c.auth.login("u", "p", remember_me=True))
    assert json.loads(portal.requests[-1][3]) == {
        "username": "u",
        "password": "p",
        "rememberMe": True,
    }

    portal.routes[LOGIN] = (
        200,
        page_method({"success": False, "msg": "Invalid Username or Password"}),
    )
    with pytest.raises(LoginRejectedError) as info:
        run(client, lambda c: c.auth.login("u", "wrong"))
    assert info.value.reason == "Invalid Username or Password"
    assert "Invalid Username or Password" in str(info.value)


def test_api_key_rejection(client: Client | AsyncClient, portal: FakePortal) -> None:
    portal.routes[GET_KEY] = (200, page_method({"error": False, "apiKey": "k-1"}))
    assert run(client, lambda c: c.account.api_key("p")) == "k-1"

    portal.routes[GET_KEY] = (
        200,
        page_method({"error": True, "msg": "Invalid password specified."}),
    )
    with pytest.raises(PasswordRejectedError) as info:
        run(client, lambda c: c.account.api_key("wrong"))
    assert info.value.reason == "Invalid password specified."


def test_http_errors_carry_status_and_body(
    client: Client | AsyncClient, portal: FakePortal
) -> None:
    portal.routes[TREE] = (401, {"Message": "Authorization has been denied"})
    with pytest.raises(HTTPStatusError) as info:
        run(client, lambda c: c.account.meters())
    assert info.value.status == 401
    assert "Authorization has been denied" in info.value.body
    assert isinstance(info.value, provident.ProvidentError)

    portal.routes[TREE] = (200, {"not": "a list"})
    with pytest.raises(DecodeError) as decode:
        run(client, lambda c: c.account.meters())
    assert decode.value.status == 200


def test_unreachable_server_is_a_transport_error() -> None:
    # Port 9 (discard) on loopback: nothing listens, so the connection is refused.
    client = Client(base_url="http://127.0.0.1:9/", connect_timeout=2)
    with pytest.raises(TransportError) as info:
        client.auth.verify()
    assert info.value.status is None
    # Connection failures on this endpoint can be reported as refusal or connect-timeout
    # depending on platform/network stack, but should always map to TransportError.
    assert isinstance(info.value.timeout, bool)


@pytest.mark.parametrize(
    ("kwargs", "message"),
    [
        ({"base_url": "mailto:x@example.com"}, "cannot be a base"),
        ({"base_url": "not a url"}, "url"),
        ({"timeout": -1}, "timeout"),
        ({"connect_timeout": float("nan")}, "connect_timeout"),
    ],
)
def test_invalid_config_is_rejected(kwargs: dict[str, Any], message: str) -> None:
    with pytest.raises(ValueError, match=message):
        Client(**kwargs)


def test_sync_calls_release_the_gil(portal: FakePortal) -> None:
    """Blocking calls from several threads overlap instead of serializing on the GIL."""
    portal.delay = 0.3
    portal.routes["/login/LoginService.aspx/CheckLogin"] = (200, {"d": True})
    client = Client(base_url=portal.url)

    started = time.monotonic()
    with ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(lambda _: client.auth.verify(), range(4)))
    elapsed = time.monotonic() - started

    assert results == [True] * 4
    assert elapsed < 2 * portal.delay, f"calls serialized: {elapsed:.2f}s"


RATES_LOGIN = "/do-rates-login"
RATES_PAGE = "/rates-and-fees"


def test_rates_login_and_fetch(client: Client | AsyncClient, portal: FakePortal) -> None:
    portal.routes[RATES_LOGIN] = rates_login_ok()
    portal.routes[RATES_PAGE] = (200, RATES_HTML, "text/html")

    run(client, lambda c: c.rates.login("874-1010", "222212"))
    rates = run(client, lambda c: c.rates.fetch())

    # The portal's credentials go in with their fields swapped.
    body = next(r[3] for r in portal.requests if r[1] == RATES_LOGIN)
    assert body == b"CustomerNumber=222212&LocationID=874-1010"

    assert rates.tou_off_peak.current == 9.8
    assert rates.tou_on_peak.previous == 15.8
    assert rates.tou_on_peak.current_effective == date(2025, 11, 1)
    assert rates.ulo_ultra_low_overnight.current == 3.9
    # ULO on-peak must not be confused with TOU on-peak (same "On-Peak" label).
    assert rates.ulo_on_peak.current == 39.1
    assert rates.tier_2.current == 14.2
    assert rates.admin_fee.current == 11.90
    assert rates.meter_fee.previous == 8.31
    assert rates.tou_off_peak.value() == 9.8
    assert rates.unknown == {}


def test_rates_login_rejected(client: Client | AsyncClient, portal: FakePortal) -> None:
    portal.routes[RATES_LOGIN] = rates_login_rejected()
    with pytest.raises(LoginRejectedError) as info:
        run(client, lambda c: c.rates.login("000-0000", "000000"))
    assert info.value.reason == "Invalid login."


def test_rates_sign_in_form_raises_session_required(
    client: Client | AsyncClient, portal: FakePortal
) -> None:
    # What the real site serves without (or after expiry of) a rates session.
    portal.routes[RATES_PAGE] = (
        200,
        '<form action="/do-rates-login" method="POST">'
        '<input name="CustomerNumber"><input name="LocationID"></form>',
        "text/html",
    )
    with pytest.raises(RatesSessionRequiredError):
        run(client, lambda c: c.rates.fetch())


def test_rates_page_drift_degrades_to_none(
    client: Client | AsyncClient, portal: FakePortal
) -> None:
    portal.routes[RATES_LOGIN] = rates_login_ok()
    portal.routes[RATES_PAGE] = (
        200,
        "<html><body><h2>Electricity Energy Rates</h2><table><tr>"
        "<td>Time-Of-Use (TOU)</td><td></td><td></td><td></td><td></td></tr>"
        "<tr><td>Off-Peak</td><td>—</td><td>9.8</td><td></td><td>—</td></tr>"
        "</table></body></html>",
        "text/html",
    )
    run(client, lambda c: c.rates.login("u", "p"))
    rates = run(client, lambda c: c.rates.fetch())
    assert rates.tou_off_peak.current == 9.8
    assert rates.tou_off_peak.previous is None
    assert rates.ulo_on_peak is None
    assert rates.admin_fee is None


def test_rates_other_sections_are_exposed_apart(
    client: Client | AsyncClient, portal: FakePortal
) -> None:
    portal.routes[RATES_LOGIN] = rates_login_ok()
    portal.routes[RATES_PAGE] = (
        200,
        "<h2>EV Charging Rates</h2><table>"
        "<tr><td>On-Peak</td><td>2025-11-01</td><td>25.0</td><td></td><td></td></tr></table>"
        + RATES_HTML,
        "text/html",
    )
    run(client, lambda c: c.rates.login("u", "p"))
    rates = run(client, lambda c: c.rates.fetch())
    assert rates.tou_on_peak.current == 20.3
    assert rates.sections["EV Charging Rates"]["On-Peak"].current == 25.0


def test_rates_fixture_matches_the_rust_one() -> None:
    """The two copies of the rates page must not drift apart (skipped outside the checkout)."""
    rust = Path(__file__).resolve().parents[2] / "tests/fixtures/rates_page.html"
    if not rust.is_file():
        pytest.skip("not run from the repository checkout")
    assert rust.read_text() == RATES_HTML

# SPDX-License-Identifier: GPL-3.0-only

"""Fixtures: a local fake portal for offline tests, and credentials for e2e tests.

E2E tests (`-m e2e`) read PROVIDENT_USERNAME / PROVIDENT_PASSWORD (and optional
PROVIDENT_BASE_URL) from the environment, else from the repository's `.env`.
"""

from __future__ import annotations

import json
import os
import threading
import time
from collections.abc import Iterator
from dataclasses import dataclass, field
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest

# (status, JSON body) or (status, text body, content type) for a request path; 404 otherwise.
Route = tuple[int, object] | tuple[int, str, str]


@dataclass
class FakePortal:
    """HTTP server answering canned JSON per path, recording each request."""

    url: str
    routes: dict[str, Route] = field(default_factory=dict)
    requests: list[tuple[str, str, dict[str, list[str]], bytes]] = field(default_factory=list)
    #: Seconds to wait before answering each request.
    delay: float = 0.0


@pytest.fixture
def portal() -> Iterator[FakePortal]:
    fake = FakePortal(url="")

    class Handler(BaseHTTPRequestHandler):
        def _answer(self) -> None:
            url = urlsplit(self.path)
            length = int(self.headers.get("Content-Length") or 0)
            fake.requests.append(
                (self.command, url.path, parse_qs(url.query), self.rfile.read(length))
            )
            route = fake.routes.get(url.path, (404, {"Message": "not found"}))
            status, body = route[0], route[1]
            ctype = route[2] if len(route) > 2 else "application/json"
            time.sleep(fake.delay)
            payload = body.encode() if isinstance(body, str) else json.dumps(body).encode()
            self.send_response(status)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        do_GET = do_POST = _answer

        def log_message(self, *args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    fake.url = f"http://127.0.0.1:{server.server_address[1]}/"
    thread = threading.Thread(
        target=server.serve_forever, kwargs={"poll_interval": 0.01}, daemon=True
    )
    thread.start()
    yield fake
    server.shutdown()
    server.server_close()


def page_method(result: object) -> object:
    """ASP.NET page-method envelope whose `d` is `result` serialized into a JSON string."""
    return {"d": json.dumps(result)}


def _dotenv() -> dict[str, str]:
    path = Path(__file__).resolve().parents[2] / ".env"
    if not path.is_file():
        return {}
    values = {}
    for line in path.read_text().splitlines():
        key, sep, value = line.strip().partition("=")
        if sep and not key.startswith("#"):
            values[key.strip()] = value.strip().strip("\"'")
    return values


def _var(name: str) -> str | None:
    return os.environ.get(name) or _dotenv().get(name) or None


@pytest.fixture
def credentials() -> tuple[str, str]:
    values = {name: _var(name) for name in ("PROVIDENT_USERNAME", "PROVIDENT_PASSWORD")}
    missing = [name for name, value in values.items() if not value]
    if missing:
        pytest.fail(f"{', '.join(missing)} must be set (env or .env) to run e2e tests")
    return values["PROVIDENT_USERNAME"], values["PROVIDENT_PASSWORD"]  # type: ignore[return-value]


@pytest.fixture
def base_url() -> str | None:
    return _var("PROVIDENT_BASE_URL")


def rates_login_ok() -> Route:
    """Successful `do-rates-login` answer."""
    return (200, {"response": "ok", "message": "", "error": 0})


def rates_login_rejected() -> Route:
    """Refused `do-rates-login` answer."""
    return (200, {"response": "no", "message": "Invalid login.", "error": 0})


#: The real rates-and-fees page (trimmed), shared with the Rust unit tests.
RATES_HTML = (Path(__file__).resolve().parents[2] / "tests/fixtures/rates_page.html").read_text()

# provident (Python)

[![PyPI](https://img.shields.io/pypi/v/provident)](https://pypi.org/project/provident/)
[![Python](https://img.shields.io/pypi/pyversions/provident)](https://pypi.org/project/provident/)
[![CI](https://github.com/dangreco/provident/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/dangreco/provident/actions/workflows/ci.yml?query=branch%3Adev)
[![License: GPL-3.0-only](https://img.shields.io/pypi/l/provident)](https://github.com/dangreco/provident/blob/dev/LICENSE)

Unofficial Python client for the [Provident](https://provident.meterconnex.com) (MeterConnex)
utility metering portal: sign in, list your meters, and fetch consumption history and
billing-period reports for electricity, water, heating, cooling, and gas.

It is a thin binding to the [`provident`](https://crates.io/crates/provident) Rust crate,
so both share one implementation and release version.

> Not affiliated with or endorsed by Provident or QMC. The portal has no public API; this
> package talks to the same endpoints its web pages use, which may change without notice.

## Installation

```sh
pip install provident
```

Wheels are prebuilt for CPython 3.11+ on Linux (x86_64, aarch64; glibc and musl), macOS
(Apple Silicon, Intel), and Windows (x64), with TLS included. No Rust toolchain is needed.

## Usage

```python
from datetime import date, timedelta

from provident import Client, Utility

client = Client()

# Sign in once; the session is shared by every later call on this client.
client.auth.login("your-login", "your-password")

# Meters on the account.
meters = client.account.meters()
for meter in meters:
    print(meter.utility, meter.serial, meter.unit)

# Hourly electricity use for one day (ranges up to 7 days are hourly, longer ones daily).
meter = next(m for m in meters if m.utility is Utility.ELECTRICITY)
day = date(2026, 10, 18)
series = client.usage.series(meter, day, day + timedelta(days=1))  # end is exclusive
print(series[0].units, series[0].data)

# Start/end readings and totals for September, keyed by utility label.
report = client.usage.report(meters, date(2026, 9, 1), date(2026, 10, 1))
for utility, rows in report.items():
    for row in rows:
        print(utility, row.serial, row.total)
```

### asyncio

`AsyncClient` has the same methods, each returning an awaitable:

```python
import asyncio

from provident import AsyncClient


async def main() -> None:
    client = AsyncClient()
    await client.auth.login("your-login", "your-password")
    meters = await client.account.meters()
    print(meters)


asyncio.run(main())
```

`Client` releases the GIL while it waits, so it is safe to use from threads.

## What's available

| Area | Calls |
|---|---|
| `client.auth` | `login(username, password, *, remember_me=False)`, `verify()` |
| `client.account` | `meters()`, `meter(id)`, `meter_by_serial(serial)`, `utilities()`, `location(node)`, `details()`, `api_key(password)` |
| `client.usage` | `series(meters, start, end, *, aggregate_groups=False)`, `report(meters, start, end, *, aggregate_groups=False, include_deactivated=False)` |

`meters`/`node` arguments take a `Meter`, a `TreeNode`, or a node id string (`"MP:692294"`,
`"GROUP:121497"`); `meters` also takes an iterable of them. A group covers every meter below
it. `Client(base_url=..., timeout=..., connect_timeout=..., user_agent=...)` overrides the
defaults (timeouts in seconds).

Results are read-only objects: `Meter`, `TreeNode`, `Series`, `ReportRow`, `AccountDetails`,
and the `Utility` string enum. The package ships type hints.

### Errors

Every exception derives from `ProvidentError`:

| Exception | When | Attributes |
|---|---|---|
| `LoginRejectedError` | Wrong username or password | `reason` |
| `PasswordRejectedError` | Wrong password for `api_key` | `reason` |
| `HTTPStatusError` | Non-2xx response, e.g. 401 without a session | `status`, `body` |
| `DecodeError` | Response in an unexpected shape | `status`, `body` |
| `TransportError` | Connection, TLS, or timeout failure | `status`, `timeout` |
| `InvalidMeterError` | A meter in the tree could not be read | |

Bad arguments (an unparsable node id, a negative timeout) raise `ValueError`/`TypeError`.

### Things worth knowing

- **Time zones.** Dates are calendar days with no zone. Hourly buckets for electricity behave
  like local time; water meters appear to use a fixed offset about two hours earlier in winter.
- **Missing readings.** Hourly series report `0.0` for missing hours; report rows flag them
  with `error` and leave values `None`.
- **API key.** A wrong password for `api_key` counts as a failed sign-in, and several in a row
  briefly lock the account out.

The [Rust crate docs](https://docs.rs/provident) cover each endpoint in more depth.

## License

[GPL-3.0-only](https://github.com/dangreco/provident/blob/dev/LICENSE).

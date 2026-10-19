# provident

[![crates.io](https://img.shields.io/crates/v/provident.svg)](https://crates.io/crates/provident)
[![docs.rs](https://img.shields.io/docsrs/provident)](https://docs.rs/provident)
[![CI](https://github.com/dangreco/provident/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/dangreco/provident/actions/workflows/ci.yml?query=branch%3Adev)
[![License: GPL-3.0-only](https://img.shields.io/crates/l/provident.svg)](https://github.com/dangreco/provident/blob/dev/LICENSE)

Unofficial Rust client for the [Provident](https://provident.meterconnex.com) (MeterConnex)
utility metering portal: sign in, list your meters, and fetch consumption history and
billing-period reports for electricity, water, heating, cooling, and gas.

> Not affiliated with or endorsed by Provident or QMC. The portal has no public API; this
> crate talks to the same endpoints its web pages use, which may change without notice.

## Installation

```toml
[dependencies]
provident = "0.1"
```

The client is async by default and runtime-agnostic (any executor that can drive
[`reqwest`](https://docs.rs/reqwest)). For a blocking client, disable default features:

```toml
[dependencies]
provident = { version = "0.1", default-features = false, features = ["blocking"] }
```

`async` and `blocking` are mutually exclusive: enable exactly one. TLS uses the platform's
native stack (OpenSSL on Linux), because the portal only offers cipher suites rustls does not
support.

## Usage

```rust,no_run
use chrono::{Days, NaiveDate};
use provident::Client;
use provident::models::api::internal::graphs::quick_graphs::QuickGraphsRequest;
use provident::models::common::Utility;
use provident::models::login::login_service::process_login::ProcessLoginRequest;

#[tokio::main]
async fn main() -> provident::Result<()> {
    let client = Client::new()?;

    // Sign in once; the session cookie is shared by every call on this client and its clones.
    let credentials = ProcessLoginRequest::builder()
        .username("your-login")
        .password("your-password")
        .build();
    client.auth.login(&credentials).await?;

    // Meters on the account.
    let meters = client.account.meters().await?;
    for meter in &meters {
        println!("{:?} meter {} ({})", meter.utility, meter.serial, meter.unit);
    }

    // Hourly electricity use for one day (ranges up to 7 days are hourly, longer ones daily).
    let meter = meters
        .iter()
        .find(|m| m.utility == Utility::Electricity)
        .expect("account has an electricity meter");
    let day = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();
    let query = QuickGraphsRequest::builder()
        .meters(meter.id)
        .start(day)
        .end(day.checked_add_days(Days::new(1)).unwrap()) // end is exclusive
        .build();
    let series = client.usage.series(&query).await?;
    println!("{} per hour: {:?}", series[0].units, series[0].data);
    Ok(())
}
```

With the `blocking` feature the same calls exist without `.await`.

## What's available

| Area | Calls | Portal endpoint |
|---|---|---|
| `client.auth` | `login`, `verify` | `LoginService.aspx` |
| `client.account` | `meters`, `meter`, `meter_by_serial`, `utilities`, `location`, `details`, `api_key` | meter tree, `AccountInfo.aspx` |
| `client.usage` | `series` (hourly/daily consumption), `report` (start/end readings and totals) | Quick Graphs, Quick Reports |

Request and response types live in [`provident::models`](https://docs.rs/provident/latest/provident/models/),
laid out to mirror the portal's URLs. Things worth knowing:

- **Time zones.** Dates are calendar days with no zone. Hourly buckets for electricity behave
  like local time; water meters appear to use a fixed offset about two hours earlier in
  winter. See the `quick_graphs` module docs for details.
- **Missing readings.** Hourly series report `0.0` for missing hours; reports flag them with
  `error` and leave values empty.
- **Deliberately not wrapped.** The dashboard's `GetUtilities`, `UpdateCard`, and
  `GetChartData` page methods intermittently answer with another account's data, so this
  crate derives utilities from the per-account meter tree instead.

## Minimum supported Rust version

Rust 1.99.1 (edition 2024).

## License

[GPL-3.0-only](https://github.com/dangreco/provident/blob/dev/LICENSE).

// SPDX-License-Identifier: GPL-3.0-only

//! Shared helpers for end-to-end (e2e) tests against the real Provident endpoint
//! (`tests/tests/*.rs`).
//!
//! Every e2e test is `#[ignore = "e2e"]` so `cargo test --workspace` stays offline. Run them with
//! `make test-e2e` (`MODE=blocking` for the blocking client), i.e.:
//!
//! ```sh
//! cargo test -p provident_tests -- --ignored
//! ```
//!
//! Environment (real variables take precedence over a `.env` file, which is searched for from
//! the crate directory upward, so the workspace-root `.env` is found):
//! - [`USERNAME_VAR`], [`PASSWORD_VAR`]: account used by tests that sign in. Required only by
//!   those tests; they panic with the missing variable's name instead of silently passing.
//! - [`BASE_URL_VAR`]: optional override of the default base URL.

use std::collections::HashMap;
use std::sync::LazyLock;

use provident::models::login::login_service::process_login::ProcessLoginRequest;
use provident::{Client, ClientConfig};

pub const USERNAME_VAR: &str = "PROVIDENT_USERNAME";
pub const PASSWORD_VAR: &str = "PROVIDENT_PASSWORD";
pub const BASE_URL_VAR: &str = "PROVIDENT_BASE_URL";

/// Fresh client (own cookie jar, so no session), honoring [`BASE_URL_VAR`].
pub fn client() -> Client {
    let config = match var(BASE_URL_VAR) {
        Some(url) => ClientConfig::builder()
            .base_url(&url)
            .unwrap_or_else(|e| panic!("invalid {BASE_URL_VAR} {url:?}: {e}"))
            .build(),
        None => ClientConfig::default(),
    };
    Client::with_config(config).expect("failed to build client")
}

/// Login request from [`USERNAME_VAR`] / [`PASSWORD_VAR`].
///
/// # Panics
/// If either variable is unset or empty.
pub fn credentials() -> ProcessLoginRequest {
    ProcessLoginRequest::builder()
        .username(required(USERNAME_VAR))
        .password(required(PASSWORD_VAR))
        .build()
}

fn required(name: &str) -> String {
    var(name).unwrap_or_else(|| panic!("{name} must be set (env or .env) to run this e2e test"))
}

/// Non-empty value of `name` from the process environment, else from `.env`.
///
/// `.env` is read into a map rather than exported with `set_var`, which is unsound while
/// parallel test threads read the environment.
fn var(name: &str) -> Option<String> {
    static DOTENV: LazyLock<HashMap<String, String>> = LazyLock::new(load_dotenv);
    std::env::var(name)
        .ok()
        .or_else(|| DOTENV.get(name).cloned())
        .filter(|value| !value.is_empty())
}

fn load_dotenv() -> HashMap<String, String> {
    match dotenvy::dotenv_iter() {
        Ok(iter) => iter
            .collect::<Result<_, _>>()
            .unwrap_or_else(|e| panic!("invalid .env: {e}")),
        Err(e) if e.not_found() => HashMap::new(),
        Err(e) => panic!("failed to read .env: {e}"),
    }
}

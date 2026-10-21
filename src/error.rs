// SPDX-License-Identifier: GPL-3.0-only

/// [`std::result::Result`] defaulting its error type to [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Top-level error returned by [`Client`](crate::Client) operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Transport, HTTP status, or (de)serialization failure.
    #[error(transparent)]
    Http(#[from] crate::http::Error),

    /// The server answered but refused the credentials.
    #[error("login rejected: {}", .message.as_deref().unwrap_or("no reason given"))]
    LoginRejected {
        /// Reason reported by the server, if any.
        message: Option<String>,
    },

    /// The server answered but refused the account password (e.g. for the API key).
    #[error("password rejected: {}", .message.as_deref().unwrap_or("no reason given"))]
    PasswordRejected {
        /// Reason reported by the server, if any.
        message: Option<String>,
    },

    /// A meter in the account's meter tree could not be read (unexpected title or utility).
    #[error(transparent)]
    InvalidMeter(#[from] crate::models::api::internal::meter_tree::InvalidMeter),

    /// The rates site served its sign-in form: there is no rates session yet, or it expired.
    /// Call [`crate::rates::Rates::login`] again.
    #[error("no rates session: sign in with rates.login first")]
    RatesSessionRequired,
}

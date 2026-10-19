// SPDX-License-Identifier: GPL-3.0-only

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Transport, HTTP status, or (de)serialization failure.
    #[error(transparent)]
    Http(#[from] provident_http::Error),

    /// The server answered but refused the credentials.
    #[error("login rejected: {}", .message.as_deref().unwrap_or("no reason given"))]
    LoginRejected { message: Option<String> },

    /// A meter in the account's meter tree could not be read (unexpected title or utility).
    #[error(transparent)]
    InvalidMeter(#[from] provident_models::api::internal::meter_tree::InvalidMeter),
}

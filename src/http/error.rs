// SPDX-License-Identifier: GPL-3.0-only

use reqwest::StatusCode;

use crate::http::constants::ERROR_BODY_PREVIEW_LIMIT;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Base URL or endpoint path could not be parsed/joined.
    #[error("invalid url: {0}")]
    Url(#[from] url::ParseError),

    /// Base URL cannot carry a path (e.g. `mailto:`).
    #[error("base url cannot be a base: {0}")]
    UrlNotBase(url::Url),

    /// Query parameters could not be encoded.
    #[error("failed to encode query: {0}")]
    Query(#[from] serde_urlencoded::ser::Error),

    /// Request body could not be serialized to JSON.
    #[error("failed to serialize request body: {0}")]
    Serialize(#[source] serde_json::Error),

    /// Response body was not valid JSON for the expected type.
    #[error("failed to deserialize response ({status}): {source}")]
    Deserialize {
        status: StatusCode,
        body: String,
        #[source]
        source: serde_json::Error,
    },

    /// Server answered with a non-2xx status.
    #[error("http {status}: {body}")]
    Status { status: StatusCode, body: String },

    /// Connection, TLS, timeout, or other transport failure.
    #[error("transport error: {0}")]
    Transport(#[from] reqwest::Error),
}

impl Error {
    /// HTTP status associated with this error, if any.
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Self::Status { status, .. } | Self::Deserialize { status, .. } => Some(*status),
            Self::Transport(e) => e.status(),
            _ => None,
        }
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Transport(e) if e.is_timeout())
    }
}

/// Lossy UTF-8 preview of a body, truncated to [`ERROR_BODY_PREVIEW_LIMIT`] bytes.
pub(crate) fn body_preview(bytes: &[u8]) -> String {
    let end = bytes.len().min(ERROR_BODY_PREVIEW_LIMIT);
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

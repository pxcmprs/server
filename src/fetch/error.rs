use actix_web::http::StatusCode;
use failure::Fail;
use humansize::{file_size_opts, FileSize};
use std::fmt;

#[derive(Fail, Debug)]
pub enum FetchError {
    FetchError(#[cause] reqwest::Error),
    IllegalHost(String),
    MaxSizeExceeded(u64, u64),
    InvalidInput,
}

/// Converts a byte count to a human readable format.
fn format_bytecount(bytes: u64) -> String {
    bytes
        .file_size(file_size_opts::BINARY)
        .unwrap_or_else(|_| format!("{} bytes", bytes))
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::FetchError(ref e) => write!(f, "an upstream fetch error occurred ({})", e),
            Self::IllegalHost(ref host) => write!(f, "illegal host `{}`", host),
            Self::MaxSizeExceeded(ref limit, ref input) => write!(
                f,
                "the input size limit of {} was exceeded (received {})",
                format_bytecount(*limit),
                format_bytecount(*input),
            ),
            Self::InvalidInput => write!(f, "unable to fetch source",),
        }
    }
}

impl FetchError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::FetchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::IllegalHost(_) => StatusCode::FORBIDDEN,
            Self::MaxSizeExceeded(_, _) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(err: reqwest::Error) -> Self {
        Self::FetchError(err)
    }
}

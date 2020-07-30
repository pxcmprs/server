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

fn format_bytes(bytes: u64) -> String {
    bytes
        .file_size(file_size_opts::BINARY)
        .unwrap_or_else(|_| format!("{} bytes", bytes))
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FetchError::FetchError(ref e) => write!(f, "an upstream fetch error occurred ({})", e),
            FetchError::IllegalHost(ref host) => write!(f, "illegal host `{}`", host),
            FetchError::MaxSizeExceeded(ref limit, ref input) => write!(
                f,
                "the input size limit of {} was exceeded, received {}",
                format_bytes(*limit),
                format_bytes(*input),
            ),
            FetchError::InvalidInput => write!(f, "unable to fetch source",),
        }
    }
}

impl FetchError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            FetchError::FetchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FetchError::IllegalHost(_) => StatusCode::FORBIDDEN,
            FetchError::MaxSizeExceeded(_, _) => StatusCode::PAYLOAD_TOO_LARGE,
            FetchError::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(err: reqwest::Error) -> FetchError {
        FetchError::FetchError(err)
    }
}

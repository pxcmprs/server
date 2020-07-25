use actix_web::http::StatusCode;
use failure::Fail;
use humansize::{file_size_opts, FileSize};
use std::fmt;

#[derive(Fail, Debug)]
pub enum CacheError {
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

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CacheError::FetchError(ref e) => write!(f, "an upstream fetch error occurred ({})", e),
            CacheError::IllegalHost(ref host) => write!(f, "illegal host `{}`", host),
            CacheError::MaxSizeExceeded(ref limit, ref input) => write!(
                f,
                "the input size limit of {} was exceeded, received {}",
                format_bytes(*limit),
                format_bytes(*input),
            ),
            CacheError::InvalidInput => write!(f, "unable to fetch source",),
        }
    }
}

impl CacheError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            CacheError::FetchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CacheError::IllegalHost(_) => StatusCode::FORBIDDEN,
            CacheError::MaxSizeExceeded(_, _) => StatusCode::PAYLOAD_TOO_LARGE,
            CacheError::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

impl From<reqwest::Error> for CacheError {
    fn from(err: reqwest::Error) -> CacheError {
        CacheError::FetchError(err)
    }
}

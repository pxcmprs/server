use actix_http::ResponseBuilder;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use failure::Fail;
use humansize::{file_size_opts, FileSize};
use pxcmprs_core::pipeline::error::PipelineError;

#[derive(Fail, Debug)]
pub enum PxcmprsError {
    #[fail(display = "unable to decode base64")]
    Base64Error,

    #[fail(display = "unable to convert base64 to valid utf-8 characters")]
    UnicodeError,

    #[fail(display = "unable to fetch source media")]
    FetchError(reqwest::Error),

    #[fail(display = "an error occurred during processing")]
    ProcessingError(PipelineError),

    #[fail(display = "size limit exceeded")]
    MaxSizeExceeded(u64, u64),

    #[fail(display = "invalid input")]
    InvalidInput,
}

impl ResponseError for PxcmprsError {
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code()).body(match self {
            PxcmprsError::FetchError(err) => format!(
                "{} ({})",
                self.to_string(),
                match err.status() {
                    Some(status) => format!("upstream status: {}", status),
                    None => "unknown error".to_string(),
                }
            ),
            PxcmprsError::MaxSizeExceeded(limit, got) => format!(
                "{} (limit: {}, got: {})",
                self.to_string(),
                limit
                    .file_size(file_size_opts::BINARY)
                    .unwrap_or(limit.to_string()),
                got.file_size(file_size_opts::BINARY)
                    .unwrap_or(got.to_string())
            ),
            _ => self.to_string(),
        })
    }

    fn status_code(&self) -> StatusCode {
        match self {
            PxcmprsError::Base64Error => StatusCode::BAD_REQUEST,
            PxcmprsError::UnicodeError => StatusCode::BAD_REQUEST,
            PxcmprsError::FetchError(err) => err.status().unwrap_or(StatusCode::BAD_GATEWAY),
            PxcmprsError::ProcessingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PxcmprsError::MaxSizeExceeded(_, _) => StatusCode::PAYLOAD_TOO_LARGE,
            PxcmprsError::InvalidInput => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<base64::DecodeError> for PxcmprsError {
    fn from(_err: base64::DecodeError) -> PxcmprsError {
        PxcmprsError::Base64Error
    }
}

impl From<std::string::FromUtf8Error> for PxcmprsError {
    fn from(_err: std::string::FromUtf8Error) -> PxcmprsError {
        PxcmprsError::UnicodeError
    }
}

impl From<reqwest::Error> for PxcmprsError {
    fn from(err: reqwest::Error) -> PxcmprsError {
        PxcmprsError::FetchError(err)
    }
}

impl From<PipelineError> for PxcmprsError {
    fn from(err: PipelineError) -> PxcmprsError {
        PxcmprsError::ProcessingError(err)
    }
}

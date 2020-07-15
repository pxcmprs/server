use super::cache::error::CacheError;
use super::transform::error::TransformError;
use actix_web::{error, http::StatusCode, HttpResponse};
use failure::Fail;
use std::string::FromUtf8Error;

#[derive(Fail, Debug)]
pub enum PxcmprsError {
    #[fail(display = "unable to parse base64")]
    Base64Error(#[cause] base64::DecodeError),
    #[fail(display = "cannot parse bytes to string as it contains invalid characters")]
    UnicodeError(#[cause] FromUtf8Error),
    #[fail(display = "a cache error occurred ({})", _0)]
    CacheError(#[cause] CacheError),
    #[fail(display = "an error occurred while transforming the media ({})", _0)]
    TransformError(#[cause] TransformError),
}

impl error::ResponseError for PxcmprsError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            PxcmprsError::Base64Error(_) => StatusCode::BAD_REQUEST,
            PxcmprsError::UnicodeError(_) => StatusCode::BAD_REQUEST,
            PxcmprsError::CacheError(err) => err.status_code(),
            PxcmprsError::TransformError(err) => err.status_code(),
        }
    }
}

impl From<base64::DecodeError> for PxcmprsError {
    fn from(err: base64::DecodeError) -> PxcmprsError {
        PxcmprsError::Base64Error(err)
    }
}

impl From<FromUtf8Error> for PxcmprsError {
    fn from(err: FromUtf8Error) -> PxcmprsError {
        PxcmprsError::UnicodeError(err)
    }
}

impl From<CacheError> for PxcmprsError {
    fn from(err: CacheError) -> PxcmprsError {
        PxcmprsError::CacheError(err)
    }
}

impl From<TransformError> for PxcmprsError {
    fn from(err: TransformError) -> PxcmprsError {
        PxcmprsError::TransformError(err)
    }
}

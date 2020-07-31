use super::fetch::error::FetchError;
use super::transform::error::TransformError;
use actix_web::{
    error,
    http::{header, StatusCode},
    HttpResponse,
};
use failure::Fail;
use std::str::Utf8Error;

#[derive(Fail, Debug)]
pub enum PxcmprsError {
    #[fail(display = "unable to parse base64")]
    Base64Error(#[cause] base64::DecodeError),
    #[fail(display = "cannot parse bytes to string as it contains invalid characters")]
    UnicodeError(#[cause] Utf8Error),
    #[fail(display = "invalid url (got: {})", _1)]
    UrlParseError(#[cause] url::ParseError, String),
    #[fail(display = "{}", _0)]
    FetchError(#[cause] FetchError),
    #[fail(display = "{}", _0)]
    TransformError(#[cause] TransformError),
}

impl error::ResponseError for PxcmprsError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .set_header(header::CONTENT_TYPE, header::ContentType::plaintext())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            PxcmprsError::Base64Error(_) => StatusCode::BAD_REQUEST,
            PxcmprsError::UnicodeError(_) => StatusCode::BAD_REQUEST,
            PxcmprsError::UrlParseError(_, _) => StatusCode::BAD_REQUEST,
            PxcmprsError::FetchError(err) => err.status_code(),
            PxcmprsError::TransformError(err) => err.status_code(),
        }
    }
}

impl From<base64::DecodeError> for PxcmprsError {
    fn from(err: base64::DecodeError) -> PxcmprsError {
        PxcmprsError::Base64Error(err)
    }
}

impl From<Utf8Error> for PxcmprsError {
    fn from(err: Utf8Error) -> PxcmprsError {
        PxcmprsError::UnicodeError(err)
    }
}

impl From<FetchError> for PxcmprsError {
    fn from(err: FetchError) -> PxcmprsError {
        PxcmprsError::FetchError(err)
    }
}

impl From<TransformError> for PxcmprsError {
    fn from(err: TransformError) -> PxcmprsError {
        PxcmprsError::TransformError(err)
    }
}

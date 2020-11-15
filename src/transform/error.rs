use crate::encoding::error::EncodeError;
use actix_web::http::StatusCode;
use failure::Fail;

#[derive(Debug, Fail)]
pub enum DecodeError {
    #[fail(display = "unknown error")]
    ImageError(#[cause] image::ImageError),

    #[fail(display = "unsupported encoding")]
    UnsupportedEncoding,
}

impl DecodeError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            DecodeError::ImageError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DecodeError::UnsupportedEncoding => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        }
    }
}

impl From<image::ImageError> for DecodeError {
    fn from(err: image::ImageError) -> DecodeError {
        DecodeError::ImageError(err)
    }
}

#[derive(Debug, Fail)]
pub enum TransformError {
    #[fail(display = "{}", _0)]
    EncodeError(#[cause] EncodeError),

    #[fail(display = "{}", _0)]
    DecodeError(#[cause] DecodeError),
}

impl TransformError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            TransformError::EncodeError(err) => err.status_code(),
            TransformError::DecodeError(err) => err.status_code(),
        }
    }
}

impl From<EncodeError> for TransformError {
    fn from(err: EncodeError) -> TransformError {
        TransformError::EncodeError(err)
    }
}

impl From<DecodeError> for TransformError {
    fn from(err: DecodeError) -> TransformError {
        TransformError::DecodeError(err)
    }
}

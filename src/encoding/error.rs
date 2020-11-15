use actix_web::http::StatusCode;
use failure::Fail;

#[derive(Debug, Fail)]
pub enum EncodeError {
    #[fail(display = "unknown error")]
    ImageError(#[cause] image::ImageError),

    #[fail(display = "unsupported encoding")]
    UnsupportedEncoding,

    #[fail(display = "invalid quality number (range: {}-{}, got: {})", _0, _1, _2)]
    InvalidQuality(u8, u8, u8),
}

impl EncodeError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            EncodeError::ImageError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            EncodeError::UnsupportedEncoding => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            EncodeError::InvalidQuality(_, _, _) => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<image::ImageError> for EncodeError {
    fn from(err: image::ImageError) -> EncodeError {
        EncodeError::ImageError(err)
    }
}

pub type EncodeResult<T> = Result<T, EncodeError>;

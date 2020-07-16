use actix_web::http::StatusCode;
use failure::Fail;

#[derive(Fail, Debug)]
pub enum CacheError {
    #[fail(display = "an upstream fetch error occurred ({})", _0)]
    FetchError(#[cause] reqwest::Error),

    #[fail(display = "illegal host `{}`", _0)]
    IllegalHost(String),
}

impl CacheError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            CacheError::FetchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CacheError::IllegalHost(_) => StatusCode::FORBIDDEN,
        }
    }
}

impl From<reqwest::Error> for CacheError {
    fn from(err: reqwest::Error) -> CacheError {
        CacheError::FetchError(err)
    }
}

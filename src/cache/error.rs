use actix_web::http::StatusCode;
use failure::Fail;

#[derive(Fail, Debug)]
pub enum CacheError {
    #[fail(display = "an upstream fetch error occurred")]
    FetchError(#[cause] reqwest::Error),
}

impl CacheError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            CacheError::FetchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<reqwest::Error> for CacheError {
    fn from(err: reqwest::Error) -> CacheError {
        CacheError::FetchError(err)
    }
}

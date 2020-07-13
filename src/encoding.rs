use actix_web::{http::header, HttpRequest};
use pxcmprs_core::spec::Encoding;

/// Detect a suitable encoding based on the incoming `Accept` HTTP header.
pub fn detect_encoding(req: &HttpRequest) -> &Encoding {
    if let Some(accept) = req.headers().get(header::ACCEPT) {
        if accept.to_str().unwrap_or("").contains("image/webp") {
            return &Encoding::WebP;
        }
    }

    &Encoding::Jpeg
}

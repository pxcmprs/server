use actix_http::ResponseBuilder;
use actix_web::{
    get,
    http::{header, StatusCode},
    web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use pxcmprs_core::pipeline::handle_query;
use pxcmprs_core::spec::{Encoding, Query, ResizeMode};
use serde::Deserialize;

/// Related to output encodings.
mod encoding;

/// Errors!
mod error;

/// Limits so that the servers don't blow up.
mod limits;

mod fetching;

use encoding::detect_encoding;
use error::PxcmprsError;
use fetching::fetch_bytes;
use limits::PxcmprsLimits;

/// Commands defined in the request path.
#[derive(Deserialize)]
struct Command {
    /// URL of the input, base64url-encoded.
    source: String,

    /// Encoding defined in the path as a file extension, e.g. `.jpeg`.
    encoding: Option<Encoding>,
}

/// Dimensions defined in the query.
#[derive(Deserialize)]
struct Dimensions {
    /// Width of the output media.
    #[serde(alias = "w")]
    width: Option<u32>,

    /// Height of the output.
    #[serde(alias = "h")]
    height: Option<u32>,
}

/// A fire in the data center is undesirable.
const LIMITS: PxcmprsLimits = PxcmprsLimits {
    max_dimensions: 4096,
    max_input_size: 1 << 25, // 32 MiB
};

/// The index page.
#[get("/")]
async fn index(req: HttpRequest) -> Result<HttpResponse> {
    let mut response = HttpResponse::build(StatusCode::OK);

    let comment = format!(
        "Pxcmprs version {}\nSyntax: /{{source}}.{{encoding}}?width&height\nDocumentation available at {}\n",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_REPOSITORY")
    );

    if let Some(accept) = req.headers().get(header::ACCEPT) {
        if accept.to_str().unwrap_or("").contains("text/html") {
            let html = format!("<!--\n{}--><body style=\"background-image: url('https://unsplash.com/photos/erApmfRX7eo/download?w=2400');background-size: cover;background-position: center;\" />", comment);

            return Ok(response.content_type("text/html; charset=utf-8").body(html));
        }
    }

    Ok(response.body(comment))
}

async fn pxcmprs(
    req: HttpRequest,
    command: web::Path<Command>,
    dimensions: web::Query<Dimensions>,
) -> Result<HttpResponse, PxcmprsError> {
    let encoding: &Encoding = match &command.encoding {
        Some(encoding) => encoding,
        None => detect_encoding(&req),
    };

    let url = String::from_utf8(base64::decode_config(
        &command.source,
        base64::URL_SAFE_NO_PAD,
    )?)?;

    let fetch_response = fetch_bytes(&url, LIMITS.max_input_size).await?;

    let source_size = fetch_response.bytes.len();

    let (width, height) = LIMITS.sanitize_dimensions(dimensions.width, dimensions.height);

    let output = handle_query(
        fetch_response.bytes,
        Query {
            encoding: *encoding,
            width: Some(width),
            height: Some(height),
            source: url,
            mode: Some(ResizeMode::Contain),
        },
    )?;

    Ok(ResponseBuilder::new(StatusCode::OK)
        .set_header(header::CONTENT_TYPE, output.mime)
        .set_header(
            "pxcmprs-download-ms",
            fetch_response.duration.as_millis().to_string(),
        )
        .set_header("pxcmprs-source-size", source_size)
        .body(output.bytes))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Listening");
    HttpServer::new(|| {
        App::new()
            .service(
                web::resource(["/{source}.{encoding}", "/{source}"]).route(web::get().to(pxcmprs)),
            )
            .service(index)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

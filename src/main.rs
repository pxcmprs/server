mod cache;
mod error;
mod transform;

use actix_web::{
    http::{header, StatusCode},
    web, App, HttpRequest, HttpResponse, HttpServer,
};
use cache::Cache;
use serde::Deserialize;
use std::time::Duration;
use transform::{
    encoding::{Encoding, Serializable as SerializableEncoding},
    error::TransformError,
};

/// Commands defined in the request path.
#[derive(Deserialize)]
struct Command {
    /// URL of the input, base64url-encoded.
    source: String,

    encoding: Option<SerializableEncoding>,
}

#[derive(Deserialize)]
struct Options {
    quality: Option<u8>,
    width: Option<u32>,
    height: Option<u32>,
}

async fn pxcmprs(
    req: HttpRequest,
    command: web::Path<Command>,
    options: web::Query<Options>,
) -> actix_web::Result<HttpResponse, error::PxcmprsError> {
    let url = String::from_utf8(base64::decode_config(
        &command.source,
        base64::URL_SAFE_NO_PAD,
    )?)?;

    let cache = req.app_data::<Cache>().unwrap();

    let response = cache.get(&url).await.unwrap();

    let encoding = command
        .encoding
        .clone()
        .map_or_else(
            || Ok(Encoding::detect(&req)),
            |serializable| serializable.to_encoding(options.quality),
        )
        .map_err(TransformError::from)?;

    let new_dimensions = (options.width, options.height);

    let output = transform::bytes(response.bytes, new_dimensions, &encoding)?;

    Ok(HttpResponse::build(StatusCode::OK)
        .set_header(header::CONTENT_TYPE, encoding.mime_type())
        .set_header("pxcmprs-upstream-cache", response.status.to_string())
        .body(output))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let cache = Cache::new(Duration::from_secs(24 * 60 * 60));

    println!("Listening");
    HttpServer::new(move || {
        App::new().app_data(cache.clone()).service(
            web::resource(["/{source}.{encoding}", "/{source}"]).route(web::get().to(pxcmprs)),
        )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

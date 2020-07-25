mod cache;
mod error;
mod settings;
mod transform;

use actix_web::{
    http::{header, StatusCode},
    web, App, HttpRequest, HttpResponse, HttpServer,
};
use cache::Cache;
use serde::Deserialize;
use std::str;
use transform::{
    encoding::{Encoding, Serializable as SerializableEncoding},
    error::TransformError,
};
use url::Url;

/// Commands defined in the request path.
#[derive(Deserialize)]
struct Command {
    /// URL of the input, base64url-encoded.
    source: String,

    encoding: Option<SerializableEncoding>,
}

#[derive(Deserialize)]
struct Options {
    #[serde(alias = "q")]
    quality: Option<u8>,

    #[serde(alias = "w")]
    width: Option<u32>,

    #[serde(alias = "h")]
    height: Option<u32>,
}

async fn pxcmprs(
    req: HttpRequest,
    command: web::Path<Command>,
    options: web::Query<Options>,
) -> actix_web::Result<HttpResponse, error::PxcmprsError> {
    let url_bytes = &base64::decode_config(&command.source, base64::URL_SAFE_NO_PAD)?;
    let url_str = str::from_utf8(url_bytes)?;
    let url = Url::parse(url_str)
        .map_err(|e| error::PxcmprsError::UrlParseError(e, url_str.to_string()))?;

    let cache = req.app_data::<Cache>().unwrap();

    let response = cache.get(&url).await?;

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
    let settings = settings::Settings::new().unwrap();

    let cache = Cache::new(settings.fetch);

    let addr = settings.server.socket_addr();

    HttpServer::new(move || {
        App::new().app_data(cache.clone()).service(
            web::resource(["/{source}.{encoding}", "/{source}"]).route(web::get().to(pxcmprs)),
        )
    })
    .bind(addr)
    .map(|op| {
        println!("Successful bind to {}", addr);
        op
    })?
    .run()
    .await
}

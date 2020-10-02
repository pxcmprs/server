use actix_web::{
    http::{header, StatusCode},
    web, App, HttpRequest, HttpResponse, HttpServer,
};
use pxcmprs_server::{
    error,
    fetch::fetch_bytes,
    settings,
    settings::Settings,
    transform::{
        self,
        encoding::{Encoding, Serializable as SerializableEncoding},
        error::TransformError,
    },
};
use serde::Deserialize;
use std::str;
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

    let fetch_settings = req.app_data::<settings::Fetch>().unwrap();
    let transform_settings = req.app_data::<settings::Transform>().unwrap();

    let bytes = fetch_bytes(&url, fetch_settings).await?;

    let encoding = command
        .encoding
        .clone()
        .map_or_else(
            || Ok(Encoding::detect(&req)),
            |serializable| serializable.to_encoding(options.quality),
        )
        .map_err(TransformError::from)?;

    let new_dimensions = (options.width, options.height);

    let output =
        transform::transform_vec(bytes, new_dimensions, &encoding, &transform_settings.limits)?;

    Ok(HttpResponse::build(StatusCode::OK)
        .set_header(header::CONTENT_TYPE, encoding.mime_type())
        .body(output))
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(r#"<html>
        <body style="background-image: url('/aHR0cHM6Ly91bnNwbGFzaC5jb20vcGhvdG9zL2JzR015QUtENWhrL2Rvd25sb2Fk'); background-size: cover; background-position: center; min-height: 100vh; margin: 0;"/>
    </html>"#)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::new().unwrap();

    let addr = settings.server.socket_addr();

    let transform_settings = settings.transform;
    let fetch_settings = settings.fetch;

    HttpServer::new(move || {
        App::new()
            .app_data(transform_settings)
            .app_data(fetch_settings.clone())
            .service(web::resource("/").route(web::get().to(index)))
            .service(
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

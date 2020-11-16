use actix_web::{http::header, HttpRequest};
use error::{EncodeError, EncodeResult};
use image::{DynamicImage, GenericImageView, ImageOutputFormat};
use mime::{Mime, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod error;

#[derive(Debug, Clone)]
pub enum Encoding {
    Jpeg(u8),
    WebP(f32),
    Png,
    Gif,
}

impl Default for Encoding {
    fn default() -> Self {
        Self::Jpeg(85)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Serializable {
    #[serde(rename = "jpeg", alias = "jpg")]
    Jpeg,

    #[serde(rename = "webp")]
    WebP,

    #[serde(rename = "png")]
    Png,

    #[serde(rename = "gif")]
    Gif,
}

impl Serializable {
    /// Convert a serializable encoding to an `Encoding`.
    pub fn to_encoding(&self, quality: Option<u8>) -> EncodeResult<Encoding> {
        let quality = quality.unwrap_or(85);

        if quality <= 100 {
            Ok(match self {
                Self::Jpeg => Encoding::Jpeg(quality),
                Self::WebP => Encoding::WebP(quality as f32),
                Self::Png => Encoding::Png,
                Self::Gif => Encoding::Gif,
            })
        } else {
            Err(EncodeError::InvalidQuality(0, 100, quality))
        }
    }
}

impl From<Encoding> for Serializable {
    fn from(encoding: Encoding) -> Self {
        match encoding {
            Encoding::Jpeg(_) => Self::Jpeg,
            Encoding::WebP(_) => Self::WebP,
            Encoding::Png => Self::Png,
            Encoding::Gif => Self::Gif,
        }
    }
}

impl Encoding {
    pub fn from_http_request(req: &HttpRequest) -> Encoding {
        if let Some(accept) = req.headers().get(header::ACCEPT) {
            if accept.to_str().unwrap_or("").contains("image/webp") {
                return Encoding::WebP(85.0);
            }
        }

        Encoding::Jpeg(85)
    }

    pub fn image_output_format(&self) -> Option<ImageOutputFormat> {
        match self {
            Self::Jpeg(ref quality) => Some(ImageOutputFormat::Jpeg(*quality)),
            Self::WebP(_) => None,
            Self::Png => Some(ImageOutputFormat::Png),
            Self::Gif => Some(ImageOutputFormat::Gif),
        }
    }

    pub fn mime_type(self) -> Mime {
        match self {
            Self::Jpeg(_) => IMAGE_JPEG,
            Self::WebP(_) => Mime::from_str("image/webp").unwrap(),
            Self::Png => IMAGE_PNG,
            Self::Gif => IMAGE_GIF,
        }
    }

    pub fn encode_dynimage(&self, image: &DynamicImage) -> EncodeResult<Vec<u8>> {
        match self {
            Self::WebP(ref quality) => {
                let (width, height) = image.dimensions();

                let encoder: webp::Encoder = match image {
                    DynamicImage::ImageRgb8(image) => {
                        webp::Encoder::from_rgb(image.as_ref(), width, height)
                    }
                    DynamicImage::ImageRgba8(image) => {
                        webp::Encoder::from_rgba(image.as_ref(), width, height)
                    }
                    _ => return Err(EncodeError::UnsupportedEncoding),
                };

                Ok(encoder.encode(*quality).to_vec())
            }
            _ => {
                let mut bytes: Vec<u8> = Vec::new();
                image.write_to(
                    &mut bytes,
                    self.image_output_format()
                        .unwrap_or_else(|| ImageOutputFormat::Jpeg(85)),
                )?;

                Ok(bytes)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serializale_to_encoding() {
        let serializable = Serializable::WebP;

        assert_eq!(
            serializable.to_encoding(Some(69)).unwrap().mime_type(),
            Encoding::WebP(69.0).mime_type()
        );
    }
}

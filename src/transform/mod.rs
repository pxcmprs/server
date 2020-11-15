pub mod error;
pub mod limit;
pub mod resize;

use crate::encoding::Encoding;
use error::{DecodeError, TransformError};
use gif::SetParameter;
use image::{DynamicImage, GenericImageView, ImageFormat, RgbaImage};

pub type TransformResult<T> = Result<T, TransformError>;

fn transform_gif(
    input_bytes: Vec<u8>,
    dimensions: (Option<u32>, Option<u32>),
    limit: (u32, u32),
) -> TransformResult<Vec<u8>> {
    let mut decoder = gif::Decoder::new(input_bytes.as_slice());
    decoder.set(gif::ColorOutput::RGBA);
    let mut decoder = decoder
        .read_info()
        .map_err(|_| DecodeError::UnsupportedEncoding)?;

    let (owidth, oheight) = (decoder.width() as u32, decoder.height() as u32);

    let (nwidth, nheight) = resize::dimensions((owidth, oheight), dimensions, limit, false);

    let mut output: Vec<u8> = Vec::new();

    {
        let mut encoder =
            gif::Encoder::new(&mut output, nwidth as u16, nheight as u16, &[]).unwrap();
        encoder.set(gif::Repeat::Infinite).unwrap();

        while let Some(frame) = decoder.read_next_frame().unwrap() {
            let rgba = match RgbaImage::from_raw(owidth, oheight, frame.buffer.to_vec()) {
                Some(buffer) => buffer,
                None => panic!(),
            };
            let resized = DynamicImage::ImageRgba8(rgba).thumbnail_exact(nwidth, nheight);
            let mut new_rgba = resized.to_rgba().to_vec();
            let (width, height) = resized.dimensions();

            let frame =
                gif::Frame::from_rgba_speed(width as u16, height as u16, &mut *new_rgba, 30);

            encoder.write_frame(&frame).unwrap();
        }
    }

    Ok(output)
}

/// Transform a byte vector to another byte vector. This function guesses the encoding based on the data and converts it to another format with new dimensions.
pub fn transform_vec(
    bytes: Vec<u8>,
    dimensions: (Option<u32>, Option<u32>),
    target: &Encoding,
    limits: &limit::DimensionLimits,
) -> TransformResult<Vec<u8>> {
    let limit = limits.get(target);

    match (
        image::guess_format(&bytes).map_err(|_| DecodeError::UnsupportedEncoding)?,
        target,
    ) {
        (ImageFormat::Gif, Encoding::Gif) => transform_gif(bytes, dimensions, limit),

        _ => {
            let dynamic_image = image::load_from_memory(&bytes).map_err(DecodeError::ImageError)?;
            let resized = resize::dynimage(dynamic_image, dimensions, limit)?;
            Ok(target.encode_dynimage(&resized)?)
        }
    }
}

pub mod encoding;
pub mod error;
pub mod resize;

use encoding::Encoding;
use error::{DecodeError, TransformError};
use gif::SetParameter;
use image::{DynamicImage, GenericImageView, ImageFormat, RgbaImage};

pub type TransformResult<T> = Result<T, TransformError>;

pub fn bytes(
    bytes: Vec<u8>,
    dimensions: (Option<u32>, Option<u32>),
    target: &Encoding,
) -> TransformResult<Vec<u8>> {
    match (
        image::guess_format(&bytes).map_err(DecodeError::ImageError)?,
        target,
    ) {
        (ImageFormat::Gif, Encoding::Gif) => {
            let mut decoder = gif::Decoder::new(bytes.as_slice());
            decoder.set(gif::ColorOutput::RGBA);
            let mut decoder = decoder.read_info().unwrap();

            let (owidth, oheight) = (decoder.width() as u32, decoder.height() as u32);

            let (nwidth, nheight) =
                resize::dimensions((owidth, oheight), dimensions, (1024, 1024), false);

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

                    let frame = gif::Frame::from_rgba_speed(
                        width as u16,
                        height as u16,
                        &mut *new_rgba,
                        30,
                    );

                    encoder.write_frame(&frame).unwrap();
                }
            }

            Ok(output)
        }
        _ => {
            let dynamic_image = image::load_from_memory(&bytes).map_err(DecodeError::ImageError)?;
            let resized = resize::dynimage(dynamic_image, dimensions, (4096, 4096))?;
            Ok(target.encode_dynimage(&resized)?)
        }
    }
}
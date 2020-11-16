use super::Dimensions;
use super::Encoding;
use serde::Deserialize;

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct DimensionLimits {
    pub jpeg: Option<Dimensions>,
    pub webp: Option<Dimensions>,
    pub png: Option<Dimensions>,
    pub gif: Option<Dimensions>,

    /// The default limit used as a fallback for the formats.
    pub default: Dimensions,
}

impl DimensionLimits {
    pub fn get(&self, encoding: &Encoding) -> Dimensions {
        match encoding {
            Encoding::Jpeg(_) => self.jpeg,
            Encoding::WebP(_) => self.webp,
            Encoding::Png => self.png,
            Encoding::Gif => self.gif,
        }
        .unwrap_or_else(|| self.default)
    }
}

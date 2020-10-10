use super::Encoding;
use serde::Deserialize;

pub type DimensionLimit = (u32, u32);

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct DimensionLimits {
    pub jpeg: Option<DimensionLimit>,
    pub webp: Option<DimensionLimit>,
    pub png: Option<DimensionLimit>,
    pub gif: Option<DimensionLimit>,

    /// The default limit used as a fallback for the formats.
    pub default: DimensionLimit,
}

impl DimensionLimits {
    pub fn get(&self, encoding: &Encoding) -> DimensionLimit {
        match encoding {
            Encoding::Jpeg(_) => self.jpeg,
            Encoding::WebP(_) => self.webp,
            Encoding::Png => self.png,
            Encoding::Gif => self.gif,
        }
        .unwrap_or_else(|| self.default)
    }
}

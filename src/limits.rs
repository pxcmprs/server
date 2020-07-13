use num::clamp;

#[derive(Debug, Copy, Clone)]
pub struct PxcmprsLimits {
    pub max_dimensions: u32,
    pub max_input_size: u64,
}

impl PxcmprsLimits {
    pub fn sanitize_dimensions(&self, width: Option<u32>, height: Option<u32>) -> (u32, u32) {
        let nwidth = clamp(
            width.unwrap_or_else(|| self.max_dimensions),
            1,
            self.max_dimensions,
        );
        let nheight = clamp(
            height.unwrap_or_else(|| self.max_dimensions),
            1,
            self.max_dimensions,
        );

        (nwidth, nheight)
    }
}

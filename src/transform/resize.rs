use super::TransformResult;
use image::{DynamicImage, GenericImageView};
use num::clamp;

/// Calculates the width and height a media should be resized to.
/// This preserves aspect ratio, and based on the `fill` parameter
/// will either fill the dimensions to fit inside the smaller constraint
/// (will overflow the specified bounds on one axis to preserve
/// aspect ratio), or will shrink so that both dimensions are
/// completely contained with in the given `width` and `height`,
/// with empty space on one axis.
pub fn dimensions(
    old: (u32, u32),
    new: (Option<u32>, Option<u32>),
    limit: (u32, u32),
    fill: bool,
) -> (u32, u32) {
    let (width, height) = old;
    let (wlimit, hlimit) = limit;
    let (nwidth, nheight) = new;

    let (nwidth, nheight) = (
        clamp(
            nwidth.unwrap_or_else(|| if nheight.is_some() { wlimit } else { width }),
            1,
            wlimit,
        ),
        clamp(
            nheight.unwrap_or_else(|| if nwidth.is_some() { hlimit } else { height }),
            1,
            hlimit,
        ),
    );

    let ratio = u64::from(width) * u64::from(nheight);
    let nratio = u64::from(nwidth) * u64::from(height);

    let use_width = if fill {
        nratio > ratio
    } else {
        nratio <= ratio
    };
    let intermediate = if use_width {
        u64::from(height) * u64::from(nwidth) / u64::from(width)
    } else {
        u64::from(width) * u64::from(nheight) / u64::from(height)
    };
    if use_width {
        if intermediate <= u64::from(::std::u32::MAX) {
            (nwidth, intermediate as u32)
        } else {
            (
                (u64::from(nwidth) * u64::from(::std::u32::MAX) / intermediate) as u32,
                ::std::u32::MAX,
            )
        }
    } else if intermediate <= u64::from(::std::u32::MAX) {
        (intermediate as u32, nheight)
    } else {
        (
            ::std::u32::MAX,
            (u64::from(nheight) * u64::from(::std::u32::MAX) / intermediate) as u32,
        )
    }
}

pub fn dynimage(
    image: DynamicImage,
    (nwidth, nheight): (Option<u32>, Option<u32>),
    limit: (u32, u32),
) -> TransformResult<DynamicImage> {
    let (nwidth, nheight) = dimensions(image.dimensions(), (nwidth, nheight), limit, false);
    Ok(image.thumbnail_exact(nwidth, nheight))
}

//! AVIF decoding via `avif-decode` (container parsing by `avif-parse`, AV1
//! decoding by `rav1d`, a Rust port of dav1d).
//!
//! **Why an extra size check:** `avif-decode` starts the AV1 decoder with
//! default settings, which place no limit on frame size. An AVIF's container
//! (`ispe`, checked by preflight) and the AV1 bitstream inside it each
//! declare a size, and a crafted file could make them disagree. Before
//! decoding, the AV1 sequence header's maximum frame size for both the colour
//! and alpha images must fit inside the container size that preflight
//! already capped. A file that fails this is refused as a dimension mismatch.
//!
//! **Threads:** `avif-decode` uses one decoder thread per CPU core. The M2
//! scheduler must therefore count an AVIF decode as using all cores.
//!
//! **Known gaps** (tracked in `docs/IMPLEMENTATION-STATUS.md`): embedded ICC
//! profiles are not returned yet, and rotation/mirroring (`irot`/`imir`) is
//! refused at preflight because this decoder does not apply it.

use avif_decode::{Decoder, Image};
use image::metadata::Orientation;
use image::{DynamicImage, ImageBuffer};

use super::{DecodeError, DecodedImage};
use crate::format::{AlphaHint, ImageFormat, ImageInfo};

const F: ImageFormat = ImageFormat::Avif;

pub(super) fn decode(bytes: &[u8], info: ImageInfo) -> Result<DecodedImage, DecodeError> {
    let mut reader = bytes;
    let container = avif_parse::read_avif(&mut reader).map_err(container_error)?;
    let colour = container.primary_item_metadata().map_err(container_error)?;
    check_frame(
        colour.max_frame_width.get(),
        colour.max_frame_height.get(),
        &info,
    )?;
    if let Some(alpha) = container.alpha_item_metadata().map_err(container_error)? {
        check_frame(
            alpha.max_frame_width.get(),
            alpha.max_frame_height.get(),
            &info,
        )?;
    }
    // The container is parsed again inside `Decoder`; free this copy first so
    // peak memory holds only one set of AV1 data.
    drop(container);

    let image = Decoder::from_avif(bytes)
        .and_then(Decoder::to_image)
        .map_err(decoder_error)?;
    let pixels = to_dynamic(image)?;

    let decoded = (pixels.width(), pixels.height());
    if decoded.0 > info.width || decoded.1 > info.height {
        return Err(DecodeError::DimensionMismatch {
            format: F,
            header: (info.width, info.height),
            decoded,
        });
    }
    // Preflight reported the largest declared size (ADR-026); from here on
    // the decoded primary image's real size and alpha are authoritative.
    let alpha = if pixels.color().has_alpha() {
        AlphaHint::Present
    } else {
        AlphaHint::Absent
    };
    Ok(DecodedImage {
        info: ImageInfo {
            width: decoded.0,
            height: decoded.1,
            alpha,
            ..info
        },
        pixels,
        // irot/imir files are refused at preflight, so no transform applies.
        orientation: Orientation::NoTransforms,
        icc_profile: None,
        // Not read from AVIF yet (status ledger); nothing to carry over.
        exif: None,
        xmp: None,
    })
}

/// The AV1 stream may not declare a larger frame than the container did.
fn check_frame(width: u32, height: u32, info: &ImageInfo) -> Result<(), DecodeError> {
    if width <= info.width && height <= info.height {
        Ok(())
    } else {
        Err(DecodeError::DimensionMismatch {
            format: F,
            header: (info.width, info.height),
            decoded: (width, height),
        })
    }
}

fn container_error(error: avif_parse::Error) -> DecodeError {
    match error {
        avif_parse::Error::Unsupported(_) => DecodeError::UnsupportedFeature(F),
        avif_parse::Error::OutOfMemory => DecodeError::LimitExceeded(F),
        _ => DecodeError::Corrupt(F),
    }
}

fn decoder_error(error: avif_decode::Error) -> DecodeError {
    match error {
        avif_decode::Error::Unsupported(_) => DecodeError::UnsupportedFeature(F),
        avif_decode::Error::Avif(inner) => container_error(inner),
        avif_decode::Error::Io(inner) if inner.kind() == std::io::ErrorKind::OutOfMemory => {
            DecodeError::LimitExceeded(F)
        }
        _ => DecodeError::Corrupt(F),
    }
}

/// Converts `avif-decode`'s pixel buffers into the `image` crate's types.
/// 10- and 12-bit AVIF arrives as 16-bit samples scaled to the full range.
fn to_dynamic(image: Image) -> Result<DynamicImage, DecodeError> {
    fn buffer<P, C>(img: imgref::ImgVec<C>) -> Result<ImageBuffer<P, Vec<P::Subpixel>>, DecodeError>
    where
        P: image::Pixel,
        P::Subpixel: bytemuck::Pod,
        C: bytemuck::Pod,
    {
        let (pixels, width, height) = img.into_contiguous_buf();
        let samples: Vec<P::Subpixel> = bytemuck::cast_slice(&pixels).to_vec();
        let width = u32::try_from(width).map_err(|_| DecodeError::Corrupt(F))?;
        let height = u32::try_from(height).map_err(|_| DecodeError::Corrupt(F))?;
        ImageBuffer::from_raw(width, height, samples).ok_or(DecodeError::Corrupt(F))
    }

    Ok(match image {
        Image::Rgb8(img) => DynamicImage::ImageRgb8(buffer(img)?),
        Image::Rgba8(img) => DynamicImage::ImageRgba8(buffer(img)?),
        Image::Rgb16(img) => DynamicImage::ImageRgb16(buffer(img)?),
        Image::Rgba16(img) => DynamicImage::ImageRgba16(buffer(img)?),
        Image::Gray8(img) => DynamicImage::ImageLuma8(buffer(img)?),
        Image::Gray16(img) => DynamicImage::ImageLuma16(buffer(img)?),
    })
}

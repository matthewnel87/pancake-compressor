//! Decoding still images into pixels, inside fixed memory limits.
//!
//! Every input goes through [`crate::format::preflight`] first, so a decoder
//! only ever sees files whose declared size is within [`ImageLimits`]. The
//! decoder is then given those same dimensions as hard limits, plus an
//! allocation cap derived from them, so a file whose body disagrees with its
//! header cannot make it allocate more.
//!
//! **Decoders:** PNG, JPEG and still WebP use the pure-Rust decoders in the
//! `image` crate. AVIF uses `avif-decode`, with an extra check that the AV1
//! stream cannot declare a larger frame than its container (see
//! [`avif`](self::avif) module notes).
//!
//! **What it does not do:** rotate pixels for EXIF orientation, convert
//! colour profiles or drop metadata. It reports the orientation and ICC
//! profile it found so the encoding steps can apply the policy in
//! `docs/07-image-format-contract.md` explicitly.

use std::fmt;
use std::io::Cursor;

use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageReader};

use crate::format::{ImageFormat, ImageInfo, InspectError, preflight};
use crate::limits::ImageLimits;

/// Extra allocation allowed beyond the pixel buffer itself, for decoder
/// bookkeeping such as row buffers, palettes and Huffman tables.
const DECODER_OVERHEAD_BYTES: u64 = 64 * 1024 * 1024;

/// Widest pixel layout a still-image decoder here may produce: 16-bit RGBA.
const MAX_BYTES_PER_PIXEL: u64 = 8;

/// A decoded still image and the facts needed to re-encode it faithfully.
#[derive(Debug)]
pub struct DecodedImage {
    /// Header facts from preflight. `width`/`height` are confirmed to match
    /// the decoded pixels.
    pub info: ImageInfo,
    /// Pixels at the source's own bit depth and channel layout (8- or 16-bit,
    /// grey or colour, with or without alpha). Not rotated.
    pub pixels: DynamicImage,
    /// EXIF orientation found in the file. Pixels have **not** been rotated;
    /// the encoder step decides whether to bake it in (docs/07).
    pub orientation: Orientation,
    /// Embedded ICC colour profile, if any, unmodified.
    pub icc_profile: Option<Vec<u8>>,
}

/// Why decoding failed. Each variant has a stable [`DecodeError::code`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Refused before decoding (unknown, unsupported, animated, too large...).
    Preflight(InspectError),
    /// The file uses a feature of its format that the decoder does not
    /// support (for example an unusual AVIF layout).
    UnsupportedFeature(ImageFormat),
    /// The decoder hit an allocation or dimension limit.
    LimitExceeded(ImageFormat),
    /// The pixel data is damaged or inconsistent.
    Corrupt(ImageFormat),
    /// The decoder produced a different size from the header.
    DimensionMismatch {
        format: ImageFormat,
        header: (u32, u32),
        decoded: (u32, u32),
    },
}

impl DecodeError {
    /// Stable machine-readable code for progress events.
    pub fn code(&self) -> &'static str {
        match self {
            DecodeError::Preflight(inner) => inner.code(),
            DecodeError::UnsupportedFeature(_) => "unsupported-image-feature",
            DecodeError::LimitExceeded(_) => "decode-limit-exceeded",
            DecodeError::Corrupt(_) => "corrupt-image-data",
            DecodeError::DimensionMismatch { .. } => "dimension-mismatch",
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Preflight(inner) => inner.fmt(f),
            DecodeError::UnsupportedFeature(format) => write!(
                f,
                "This {format} uses a feature that Pancake Compressor cannot open yet."
            ),
            DecodeError::LimitExceeded(format) => write!(
                f,
                "This {format} needs more memory to open than Pancake Compressor allows for one image."
            ),
            DecodeError::Corrupt(format) => {
                write!(f, "This {format} file is damaged and could not be opened.")
            }
            DecodeError::DimensionMismatch { format, .. } => write!(
                f,
                "This {format} file contradicts itself about its size, so it was not opened."
            ),
        }
    }
}

impl std::error::Error for DecodeError {}

impl From<InspectError> for DecodeError {
    fn from(inner: InspectError) -> Self {
        DecodeError::Preflight(inner)
    }
}

/// Decodes a still PNG, JPEG, WebP or AVIF held in memory.
///
/// Steps: preflight (format, animation, pixel caps) → decoder with limits
/// set to the header's own dimensions → confirm decoded size equals the
/// header. Errors never include file content or paths.
pub fn decode(bytes: &[u8], limits: &ImageLimits) -> Result<DecodedImage, DecodeError> {
    let info = preflight(bytes, limits)?;
    let format = match info.format {
        ImageFormat::Png => image::ImageFormat::Png,
        ImageFormat::Jpeg => image::ImageFormat::Jpeg,
        ImageFormat::WebP => image::ImageFormat::WebP,
        ImageFormat::Avif => return avif::decode(bytes, info),
    };

    // The decoder may not exceed what the header promised. Preflight already
    // bounded width * height, so this product cannot overflow a u64.
    let pixel_bytes = u64::from(info.width) * u64::from(info.height) * MAX_BYTES_PER_PIXEL;
    let mut decoder_limits = image::Limits::default();
    decoder_limits.max_image_width = Some(info.width);
    decoder_limits.max_image_height = Some(info.height);
    decoder_limits.max_alloc = Some(pixel_bytes.saturating_add(DECODER_OVERHEAD_BYTES));

    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(decoder_limits);
    let mut decoder = reader
        .into_decoder()
        .map_err(|error| classify(info.format, &error))?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let icc_profile = decoder.icc_profile().ok().flatten();
    let pixels =
        DynamicImage::from_decoder(decoder).map_err(|error| classify(info.format, &error))?;

    let decoded = (pixels.width(), pixels.height());
    if decoded != (info.width, info.height) {
        return Err(DecodeError::DimensionMismatch {
            format: info.format,
            header: (info.width, info.height),
            decoded,
        });
    }

    Ok(DecodedImage {
        info,
        pixels,
        orientation,
        icc_profile,
    })
}

/// Maps the `image` crate's error to our stable categories. Its detailed
/// message is intentionally dropped: it is not written for end users.
fn classify(format: ImageFormat, error: &image::ImageError) -> DecodeError {
    match error {
        image::ImageError::Limits(_) => DecodeError::LimitExceeded(format),
        _ => DecodeError::Corrupt(format),
    }
}

mod avif;

#[cfg(test)]
mod tests;

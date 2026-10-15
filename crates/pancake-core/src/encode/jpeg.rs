//! JPEG output with MozJPEG.
//!
//! JPEG is always lossy here, even at quality 100 (docs/07); lossless JPEG
//! transforms are out of scope for v0.1.
//!
//! Policy applied before encoding, and reported in [`Adjustments`]:
//! - **Orientation:** EXIF orientation is applied to the pixels and no
//!   orientation tag is written, so the photo is upright in every viewer and
//!   cannot be rotated twice.
//! - **Transparency:** JPEG has no alpha. Transparent and semi-transparent
//!   pixels are composited onto [`JpegOptions::matte`] (white by default) and
//!   `transparency_flattened` is set so the UI can warn.
//! - **Precision:** 16-bit sources are reduced to 8 bits.
//! - **Colour profile:** the source ICC profile is written as standard
//!   `ICC_PROFILE` APP2 segments. EXIF/XMP are not carried yet (PNK-013).
//!
//! **Panics:** MozJPEG reports libjpeg errors by unwinding a panic through
//! its C code, and its documentation requires `catch_unwind` around every
//! call. All MozJPEG use is inside one `catch_unwind`, which is why release
//! builds use `panic = "unwind"` (ADR-030).

use std::panic::{AssertUnwindSafe, catch_unwind};

use image::{DynamicImage, GenericImageView};
use mozjpeg::{ColorSpace, Compress, Marker};

use super::{Adjustments, EncodeError, Encoded, has_transparency, upright};
use crate::decode::{DecodedImage, decode};
use crate::format::ImageFormat;
use crate::limits::ImageLimits;

const F: ImageFormat = ImageFormat::Jpeg;

/// Chroma subsampling: how much colour detail is stored relative to
/// brightness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subsampling {
    /// Colour at quarter resolution. Smallest; standard for photos.
    Quarter,
    /// Colour at full resolution. Larger; keeps fine coloured edges and text
    /// sharp.
    Full,
}

/// Settings for [`encode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpegOptions {
    /// 1 (smallest) to 100 (best).
    pub quality: u8,
    pub subsampling: Subsampling,
    /// Background colour for transparent areas.
    pub matte: [u8; 3],
}

impl Default for JpegOptions {
    /// Balanced preset: quality 80 with quarter-resolution colour, a common
    /// web setting; white matte as docs/07 specifies.
    fn default() -> Self {
        Self {
            quality: 80,
            subsampling: Subsampling::Quarter,
            matte: [255, 255, 255],
        }
    }
}

/// Encodes a decoded image as a progressive, optimised JPEG.
pub fn encode(decoded: &DecodedImage, options: JpegOptions) -> Result<Encoded, EncodeError> {
    if !(1..=100).contains(&options.quality) {
        return Err(EncodeError::InvalidOptions("JPEG quality must be 1-100"));
    }

    if let Some(profile) = &decoded.icc_profile
        && icc_segments(profile).is_empty()
    {
        // Larger than JPEG can carry (255 segments, about 16 MB). Refuse
        // rather than silently dropping the colour profile (ADR-011).
        return Err(EncodeError::Failed(F));
    }

    let (pixels, orientation_applied) = upright(decoded);
    let transparency_flattened = has_transparency(&pixels);
    let precision_reduced = matches!(
        pixels.color().bytes_per_pixel() / pixels.color().channel_count(),
        2..
    );
    let grey = !pixels.color().has_color();
    let samples = flatten(&pixels, options.matte, grey);
    let (width, height) = pixels.dimensions();

    let bytes = run_mozjpeg(
        &samples,
        width,
        height,
        grey,
        options,
        decoded.icc_profile.as_deref(),
    )?;
    verify(&bytes, width, height, &samples, grey)?;
    Ok(Encoded {
        bytes,
        adjustments: Adjustments {
            orientation_applied,
            transparency_flattened,
            precision_reduced,
            colour_profile_dropped: false,
        },
    })
}

/// 8-bit RGB (or grey) samples with alpha composited onto `matte`.
fn flatten(image: &DynamicImage, matte: [u8; 3], grey: bool) -> Vec<u8> {
    let rgba = image.to_rgba8();
    let blend = |colour: u8, background: u8, alpha: u8| -> u8 {
        let (c, b, a) = (u32::from(colour), u32::from(background), u32::from(alpha));
        // Rounded integer "over" compositing; result is always <= 255.
        ((c * a + b * (255 - a) + 127) / 255) as u8
    };
    let mut out = Vec::with_capacity(rgba.len() / 4 * if grey { 1 } else { 3 });
    for p in rgba.pixels() {
        let [r, g, b, a] = p.0;
        if grey {
            // Grey sources have r == g == b; matte is reduced to its average.
            let m = ((u16::from(matte[0]) + u16::from(matte[1]) + u16::from(matte[2])) / 3) as u8;
            out.push(blend(r, m, a));
        } else {
            out.extend([
                blend(r, matte[0], a),
                blend(g, matte[1], a),
                blend(b, matte[2], a),
            ]);
        }
    }
    out
}

fn run_mozjpeg(
    samples: &[u8],
    width: u32,
    height: u32,
    grey: bool,
    options: JpegOptions,
    icc_profile: Option<&[u8]>,
) -> Result<Vec<u8>, EncodeError> {
    let result = catch_unwind(AssertUnwindSafe(|| -> std::io::Result<Vec<u8>> {
        let mut compress = Compress::new(if grey {
            ColorSpace::JCS_GRAYSCALE
        } else {
            ColorSpace::JCS_RGB
        });
        compress.set_size(width as usize, height as usize);
        compress.set_quality(f32::from(options.quality));
        compress.set_progressive_mode();
        compress.set_optimize_coding(true);
        if !grey {
            let colour = match options.subsampling {
                Subsampling::Quarter => (2, 2),
                Subsampling::Full => (1, 1),
            };
            compress.set_chroma_sampling_pixel_sizes(colour, colour);
        }
        let mut started = compress.start_compress(Vec::new())?;
        if let Some(profile) = icc_profile {
            for segment in icc_segments(profile) {
                started.write_marker(Marker::APP(2), &segment);
            }
        }
        started.write_scanlines(samples)?;
        started.finish()
    }));
    match result {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(_)) | Err(_) => Err(EncodeError::Failed(F)),
    }
}

/// Splits an ICC profile into APP2 payloads per the ICC specification
/// (annex B.4): `"ICC_PROFILE\0"`, 1-based sequence number, segment count,
/// then up to 65,519 bytes of profile data.
fn icc_segments(profile: &[u8]) -> Vec<Vec<u8>> {
    const HEADER: &[u8] = b"ICC_PROFILE\0";
    const MAX_DATA: usize = 65_519;
    let chunks: Vec<&[u8]> = profile.chunks(MAX_DATA).collect();
    // A profile needing more than 255 segments (~16 MB) cannot be stored;
    // an empty result tells the caller to refuse rather than drop it.
    let Ok(count) = u8::try_from(chunks.len()) else {
        return Vec::new();
    };
    chunks
        .iter()
        .zip(1..=count)
        .map(|(data, number)| {
            let mut segment = HEADER.to_vec();
            segment.extend([number, count]);
            segment.extend_from_slice(data);
            segment
        })
        .collect()
}

/// Re-decodes the output: it must be a JPEG of the expected size and close
/// to the pixels that were encoded (a coarse check that catches garbage, not
/// a quality metric).
fn verify(
    bytes: &[u8],
    width: u32,
    height: u32,
    samples: &[u8],
    grey: bool,
) -> Result<(), EncodeError> {
    let fail = EncodeError::VerificationFailed(F);
    let reopened = decode(bytes, &ImageLimits::default()).map_err(|_| fail.clone())?;
    if reopened.info.format != F || (reopened.info.width, reopened.info.height) != (width, height) {
        return Err(fail);
    }
    let decoded: Vec<u8> = if grey {
        reopened.pixels.to_luma8().into_raw()
    } else {
        reopened.pixels.to_rgb8().into_raw()
    };
    if decoded.len() != samples.len() {
        return Err(fail);
    }
    let total: u64 = decoded
        .iter()
        .zip(samples)
        .map(|(a, b)| u64::from(a.abs_diff(*b)))
        .sum();
    let mean = total / samples.len().max(1) as u64;
    // Even quality 1 stays well under this on real images; garbage does not.
    if mean > 48 {
        return Err(fail);
    }
    Ok(())
}

#[cfg(test)]
mod tests;

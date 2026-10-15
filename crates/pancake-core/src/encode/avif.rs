//! Still AVIF output with `ravif` (AV1 encoding by `rav1e`).
//!
//! AVIF output is **lossy only**: ravif converts to YCbCr, so even quality
//! 100 is not pixel-identical. It is never described as lossless.
//!
//! - **Transparency** is kept (AVIF has an alpha plane). ravif's default
//!   "clean" alpha mode replaces the invisible colour of fully transparent
//!   pixels with a flat colour to save space; visible pixels are unaffected.
//! - **Bit depth:** ravif encodes 10-bit AV1 by default, which gives better
//!   quality per byte; 8- and 16-bit sources are both accepted.
//! - **Orientation** is applied to the pixels, as for JPEG and WebP.
//! - **Colour profile:** ravif cannot embed an ICC profile. A source with a
//!   profile is refused with [`EncodeError::ColourProfileNotSupported`]
//!   unless [`AvifOptions::allow_colour_profile_loss`] is set, in which case
//!   `colour_profile_dropped` is reported (docs/07: explain and ask, never
//!   discard silently). Proper conversion to sRGB is PNK-013 work.
//! - **Speed:** AV1 encoding is slow. `threads` defaults to 1 so the batch
//!   engine (M2) decides how much of the machine one AVIF may use.

use image::GenericImageView;
use ravif::{Encoder, Img, RGBA8};

use super::{Adjustments, EncodeError, Encoded, has_transparency, upright};
use crate::decode::{DecodedImage, decode};
use crate::format::ImageFormat;
use crate::limits::ImageLimits;

const F: ImageFormat = ImageFormat::Avif;

/// Settings for [`encode`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvifOptions {
    /// Colour quality, 1-100.
    pub quality: f32,
    /// Alpha quality, 1-100. Only used when the image has transparency.
    pub alpha_quality: f32,
    /// 1 (slowest, smallest) to 10 (fastest).
    pub speed: u8,
    /// Encoder threads (at least 1).
    pub threads: usize,
    /// Encode even though the source's ICC profile will be lost.
    pub allow_colour_profile_loss: bool,
}

impl Default for AvifOptions {
    /// Balanced preset: quality 70 (AVIF quality numbers run lower than JPEG
    /// for similar appearance), alpha 90, speed 6.
    fn default() -> Self {
        Self {
            quality: 70.0,
            alpha_quality: 90.0,
            speed: 6,
            threads: 1,
            allow_colour_profile_loss: false,
        }
    }
}

/// Encodes a decoded image as a still AVIF.
pub fn encode(decoded: &DecodedImage, options: AvifOptions) -> Result<Encoded, EncodeError> {
    let in_range = |q: f32| (1.0..=100.0).contains(&q);
    if !in_range(options.quality)
        || !in_range(options.alpha_quality)
        || !(1..=10).contains(&options.speed)
        || options.threads == 0
    {
        return Err(EncodeError::InvalidOptions(
            "AVIF quality 1-100, speed 1-10 and at least one thread are required",
        ));
    }
    let colour_profile_dropped = decoded.icc_profile.is_some();
    if colour_profile_dropped && !options.allow_colour_profile_loss {
        return Err(EncodeError::ColourProfileNotSupported(F));
    }

    let (pixels, orientation_applied) = upright(decoded);
    let alpha = has_transparency(&pixels);
    let (width, height) = pixels.dimensions();
    let rgba = pixels.to_rgba8();
    let buffer: Vec<RGBA8> = rgba
        .pixels()
        .map(|p| RGBA8::new(p[0], p[1], p[2], p[3]))
        .collect();

    let encoder = Encoder::new()
        .with_quality(options.quality)
        .with_alpha_quality(options.alpha_quality)
        .with_speed(options.speed)
        .with_num_threads(Some(options.threads));
    let img = Img::new(&buffer[..], width as usize, height as usize);
    // encode_rgba drops a fully opaque alpha channel by itself.
    let bytes = encoder
        .encode_rgba(img)
        .map_err(|_| EncodeError::Failed(F))?
        .avif_file;

    verify(&bytes, &rgba, alpha)?;
    Ok(Encoded {
        bytes,
        adjustments: Adjustments {
            orientation_applied,
            transparency_flattened: false,
            // 16-bit sources are passed to ravif as 8-bit and re-quantised to
            // 10-bit, so precision beyond 8 bits is not kept.
            precision_reduced: pixels.color().bytes_per_pixel() / pixels.color().channel_count()
                >= 2,
            colour_profile_dropped,
        },
    })
}

fn verify(bytes: &[u8], expected: &image::RgbaImage, alpha: bool) -> Result<(), EncodeError> {
    let fail = EncodeError::VerificationFailed(F);
    let reopened = decode(bytes, &ImageLimits::default()).map_err(|_| fail.clone())?;
    if reopened.info.format != F
        || (reopened.info.width, reopened.info.height) != expected.dimensions()
    {
        return Err(fail);
    }
    if alpha && !has_transparency(&reopened.pixels) {
        return Err(fail);
    }
    let actual = reopened.pixels.to_rgba8();
    // Compare visible colour only: ravif may repaint fully transparent pixels.
    let (mut total, mut count) = (0u64, 0u64);
    for (a, e) in actual.pixels().zip(expected.pixels()) {
        if e[3] == 0 {
            continue;
        }
        total += (0..4).map(|i| u64::from(a[i].abs_diff(e[i]))).sum::<u64>();
        count += 4;
    }
    // Coarse garbage check, not a quality metric.
    if count > 0 && total / count > 48 {
        return Err(fail);
    }
    Ok(())
}

#[cfg(test)]
mod tests;

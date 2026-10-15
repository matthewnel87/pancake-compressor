//! PNG output: lossless recompression and lossy palette reduction.
//!
//! **Lossless** ([`optimise_lossless`]) runs `oxipng` on the original file
//! bytes. It may change filters, compression and bit depth/colour type only
//! where that is exactly reversible. Settings that would alter pixels are
//! kept off: `optimize_alpha` (which rewrites the colour of fully transparent
//! pixels) and 16-to-8-bit scaling. All ancillary chunks are kept (ADR-011:
//! no silent metadata loss). The result is decoded and must match the source
//! pixel for pixel.
//!
//! **Lossy** ([`quantise_lossy`]) reduces the image to at most 256 colours
//! with `imagequant` (the library behind pngquant, GPL-3.0-or-later; the
//! reason the project is GPL, ADR-005), writes an indexed PNG and then
//! compresses it losslessly with `oxipng`. It is never described as lossless
//! or "visually lossless" (docs/07). 16-bit sources are reduced to 8 bits.
//! The ICC profile is carried over; other metadata is not yet (PNK-013).

use std::borrow::Cow;

use oxipng::{Options, StripChunks};

use super::{EncodeError, has_transparency};
use crate::decode::{DecodedImage, decode};
use crate::format::ImageFormat;
use crate::limits::ImageLimits;

const F: ImageFormat = ImageFormat::Png;

/// Settings for [`optimise_lossless`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LosslessOptions {
    /// `oxipng` preset, 0 (fastest) to 6 (smallest). Higher levels try more
    /// filter and compression combinations; output is pixel-identical at
    /// every level.
    pub effort: u8,
}

impl Default for LosslessOptions {
    fn default() -> Self {
        // oxipng's own default preset: a good size/time balance.
        Self { effort: 2 }
    }
}

/// Settings for [`quantise_lossy`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LossyOptions {
    /// Refuse to produce a result below this quality (0-100, imagequant's
    /// scale). Prevents badly banded output on photographic images.
    pub min_quality: u8,
    /// Quality imagequant aims for (0-100, at least `min_quality`).
    pub target_quality: u8,
    /// 1 (slowest, best) to 10 (fastest).
    pub speed: u8,
    /// Floyd-Steinberg dithering strength, 0.0 (none) to 1.0 (full).
    pub dithering: f32,
}

impl Default for LossyOptions {
    /// Balanced preset, close to pngquant's commonly recommended range.
    fn default() -> Self {
        Self {
            min_quality: 65,
            target_quality: 80,
            speed: 4,
            dithering: 1.0,
        }
    }
}

/// Recompresses a PNG without changing any pixel.
///
/// `source` must be the original PNG file bytes (preflight-checked by the
/// caller via [`decode`]). Returns the new bytes, which may be the same size
/// or larger than the original; choosing whether to keep them is the
/// caller's policy (ADR-015).
pub fn optimise_lossless(
    source: &[u8],
    decoded: &DecodedImage,
    options: LosslessOptions,
) -> Result<Vec<u8>, EncodeError> {
    if options.effort > 6 {
        return Err(EncodeError::InvalidOptions("PNG effort must be 0-6"));
    }
    let output = oxipng::optimize_from_memory(source, &lossless_settings(options.effort, decoded))
        .map_err(map_oxipng_error)?;
    verify(&output, decoded, Exactness::Identical)?;
    Ok(output)
}

/// Reduces an image to an indexed (palette) PNG of at most 256 colours.
///
/// Works from decoded pixels, so the source can be any supported format.
/// Returns [`EncodeError::QualityNotReached`] when `min_quality` cannot be
/// met, so the caller can fall back to a lossless path.
pub fn quantise_lossy(
    decoded: &DecodedImage,
    options: LossyOptions,
) -> Result<Vec<u8>, EncodeError> {
    validate(&options)?;
    let rgba = decoded.pixels.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels: Vec<imagequant::RGBA> = rgba
        .pixels()
        .map(|p| imagequant::RGBA::new(p[0], p[1], p[2], p[3]))
        .collect();

    let mut attributes = imagequant::new();
    attributes
        .set_speed(i32::from(options.speed))
        .and_then(|()| attributes.set_quality(options.min_quality, options.target_quality))
        .map_err(|_| EncodeError::InvalidOptions("imagequant rejected the settings"))?;
    let mut image = attributes
        .new_image(pixels, width as usize, height as usize, 0.0)
        .map_err(map_imagequant_error)?;
    let mut result = attributes
        .quantize(&mut image)
        .map_err(map_imagequant_error)?;
    result
        .set_dithering_level(options.dithering)
        .map_err(|_| EncodeError::InvalidOptions("dithering must be 0.0-1.0"))?;
    let (palette, indices) = result.remapped(&mut image).map_err(map_imagequant_error)?;

    let indexed = write_indexed(
        width,
        height,
        &palette,
        &indices,
        decoded.icc_profile.as_deref(),
    )?;
    // Indexed PNGs are compressed losslessly afterwards; effort 2 as default.
    let output = oxipng::optimize_from_memory(&indexed, &lossless_settings(2, decoded))
        .map_err(map_oxipng_error)?;
    verify(&output, decoded, Exactness::SameShape)?;
    Ok(output)
}

fn validate(options: &LossyOptions) -> Result<(), EncodeError> {
    if options.min_quality > options.target_quality || options.target_quality > 100 {
        return Err(EncodeError::InvalidOptions(
            "PNG quality must satisfy min <= target <= 100",
        ));
    }
    if !(1..=10).contains(&options.speed) {
        return Err(EncodeError::InvalidOptions("PNG speed must be 1-10"));
    }
    if !(0.0..=1.0).contains(&options.dithering) {
        return Err(EncodeError::InvalidOptions("dithering must be 0.0-1.0"));
    }
    Ok(())
}

fn lossless_settings(effort: u8, decoded: &DecodedImage) -> Options {
    let mut settings = Options::from_preset(effort);
    // Pixel-preserving only; see module docs.
    settings.optimize_alpha = false;
    settings.scale_16 = false;
    settings.strip = StripChunks::None;
    settings.force = false;
    settings.fix_errors = false;
    // Bound oxipng's own inflate step to what the image can legitimately
    // need: 16-bit RGBA plus one filter byte per row, with 1 MiB headroom.
    let width = u64::from(decoded.info.width);
    let height = u64::from(decoded.info.height);
    let raw = (width * 8 + 1) * height + 1024 * 1024;
    settings.max_decompressed_size = usize::try_from(raw).ok();
    settings
}

fn write_indexed(
    width: u32,
    height: u32,
    palette: &[imagequant::RGBA],
    indices: &[u8],
    icc_profile: Option<&[u8]>,
) -> Result<Vec<u8>, EncodeError> {
    let rgb: Vec<u8> = palette.iter().flat_map(|c| [c.r, c.g, c.b]).collect();
    // tRNS lists alpha per palette entry; trailing opaque entries can be
    // omitted, and if every entry is opaque the chunk is not needed.
    let mut alpha: Vec<u8> = palette.iter().map(|c| c.a).collect();
    while alpha.last() == Some(&255) {
        alpha.pop();
    }

    let mut info = ::png::Info::with_size(width, height);
    info.color_type = ::png::ColorType::Indexed;
    info.bit_depth = ::png::BitDepth::Eight;
    info.palette = Some(Cow::Owned(rgb));
    if !alpha.is_empty() {
        info.trns = Some(Cow::Owned(alpha));
    }
    if let Some(profile) = icc_profile {
        info.icc_profile = Some(Cow::Borrowed(profile));
    }

    let mut out = Vec::new();
    let mut writer = ::png::Encoder::with_info(&mut out, info)
        .and_then(::png::Encoder::write_header)
        .map_err(|_| EncodeError::Failed(F))?;
    writer
        .write_image_data(indices)
        .and_then(|()| writer.finish())
        .map_err(|_| EncodeError::Failed(F))?;
    Ok(out)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Exactness {
    /// Every pixel, including alpha, must match (compared at 16 bits so bit
    /// depth reductions are allowed only when exact).
    Identical,
    /// Same dimensions, and transparency must survive if the source had any.
    SameShape,
}

fn verify(output: &[u8], source: &DecodedImage, exactness: Exactness) -> Result<(), EncodeError> {
    let fail = EncodeError::VerificationFailed(F);
    let reopened = decode(output, &ImageLimits::default()).map_err(|_| fail.clone())?;
    if reopened.info.format != F
        || (reopened.info.width, reopened.info.height) != (source.info.width, source.info.height)
    {
        return Err(fail);
    }
    match exactness {
        Exactness::Identical => {
            if reopened.pixels.to_rgba16() != source.pixels.to_rgba16() {
                return Err(fail);
            }
        }
        Exactness::SameShape => {
            if has_transparency(&source.pixels) && !has_transparency(&reopened.pixels) {
                return Err(fail);
            }
        }
    }
    Ok(())
}

fn map_oxipng_error(error: oxipng::PngError) -> EncodeError {
    match error {
        oxipng::PngError::C2PAMetadataPreventsChanges
        | oxipng::PngError::ChunkPreventsChanges(_) => EncodeError::ProtectedContent(F),
        _ => EncodeError::Failed(F),
    }
}

fn map_imagequant_error(error: imagequant::Error) -> EncodeError {
    match error {
        imagequant::Error::QualityTooLow => EncodeError::QualityNotReached(F),
        _ => EncodeError::Failed(F),
    }
}

#[cfg(test)]
mod tests;

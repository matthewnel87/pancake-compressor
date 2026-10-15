//! Still WebP output with libwebp (via the `webp` crate).
//!
//! - **Lossless** keeps every pixel, including the colour of fully
//!   transparent pixels (libwebp's `exact` setting), and is verified pixel
//!   for pixel after re-decoding.
//! - **Lossy** keeps transparency (WebP has an alpha channel) at full alpha
//!   quality; colour is lossy and is never described as lossless.
//! - **Orientation and precision** follow the same policy as JPEG: EXIF
//!   orientation is applied to the pixels (no EXIF is written), 16-bit
//!   sources are reduced to 8 bits, and both are reported in [`Adjustments`].
//! - **Colour profile, EXIF and XMP:** libwebp's simple encoder writes none
//!   of them, so [`embed_metadata`] adds the extended-format `VP8X` header
//!   and `ICCP`, `EXIF` and `XMP ` chunks to its output in the order the
//!   RIFF container spec requires
//!   (<https://developers.google.com/speed/webp/docs/riff_container>). EXIF
//!   orientation is reset to upright. If the container cannot be rebuilt the
//!   encode fails rather than dropping anything.
//!
//! The `webp` crate's convenience `encode()` calls `unwrap()` internally, so
//! only `encode_advanced` with a configuration built here is used.

use image::GenericImageView;
use webp::{Encoder, WebPConfig};

use super::{
    Adjustments, Carried, EncodeError, Encoded, carry_metadata, has_transparency, upright,
};
use crate::decode::{DecodedImage, decode};
use crate::format::ImageFormat;
use crate::limits::ImageLimits;

const F: ImageFormat = ImageFormat::WebP;

/// Settings for [`encode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebpOptions {
    /// Pixel-identical output when `true`.
    pub lossless: bool,
    /// Lossy: visual quality 0-100. Lossless: compression effort 0-100
    /// (higher is smaller and slower; pixels are identical either way).
    pub quality: u8,
    /// Speed/size trade-off, 0 (fastest) to 6 (smallest).
    pub method: u8,
}

impl Default for WebpOptions {
    /// Balanced lossy preset: quality 80, method 4 (libwebp's default).
    fn default() -> Self {
        Self {
            lossless: false,
            quality: 80,
            method: 4,
        }
    }
}

/// Encodes a decoded image as a still WebP.
pub fn encode(decoded: &DecodedImage, options: WebpOptions) -> Result<Encoded, EncodeError> {
    if options.quality > 100 || options.method > 6 {
        return Err(EncodeError::InvalidOptions(
            "WebP quality must be 0-100 and method 0-6",
        ));
    }

    let (pixels, orientation_applied) = upright(decoded);
    let precision_reduced = pixels.color().bytes_per_pixel() / pixels.color().channel_count() >= 2;
    let alpha = has_transparency(&pixels);
    let (width, height) = pixels.dimensions();
    let rgba = pixels.to_rgba8();

    let mut config = WebPConfig::new().map_err(|()| EncodeError::Failed(F))?;
    config.lossless = i32::from(options.lossless);
    config.quality = f32::from(options.quality);
    config.method = i32::from(options.method);
    config.alpha_quality = 100;
    // Keep RGB under fully transparent pixels: needed for a truly
    // pixel-identical lossless result.
    config.exact = i32::from(options.lossless);
    // One thread: the batch engine decides parallelism (M2).
    config.thread_level = 0;

    let encoded = if alpha {
        Encoder::from_rgba(rgba.as_raw(), width, height).encode_advanced(&config)
    } else {
        // No transparency: encode RGB so libwebp writes no alpha data.
        let rgb = pixels.to_rgb8();
        Encoder::from_rgb(rgb.as_raw(), width, height).encode_advanced(&config)
    };
    let bytes = encoded
        .map(|memory| memory.to_vec())
        .map_err(|_| EncodeError::Failed(F))?;
    // RIFF chunk sizes are 32-bit; carry_metadata enforces that limit.
    let carried = carry_metadata(decoded, Some(u32::MAX as usize), Some(u32::MAX as usize));
    let adjustments = Adjustments {
        orientation_applied,
        transparency_flattened: false,
        precision_reduced,
        colour_profile_dropped: false,
        metadata_dropped: carried.dropped,
    };
    finish(bytes, decoded, &carried, &rgba, options, adjustments, alpha)
}

fn finish(
    bytes: Vec<u8>,
    decoded: &DecodedImage,
    carried: &Carried,
    expected: &image::RgbaImage,
    options: WebpOptions,
    adjustments: Adjustments,
    alpha: bool,
) -> Result<Encoded, EncodeError> {
    let extras = Extras {
        icc: decoded.icc_profile.as_deref(),
        exif: carried.exif.as_deref(),
        xmp: carried.xmp.as_deref(),
    };
    let bytes = if extras.is_empty() {
        bytes
    } else {
        embed_metadata(&bytes, extras, expected.width(), expected.height(), alpha)
            .ok_or(EncodeError::Failed(F))?
    };
    verify(&bytes, expected, options.lossless, alpha)?;
    Ok(Encoded { bytes, adjustments })
}

const VP8X_ICC: u8 = 0x20;
const VP8X_ALPHA: u8 = 0x10;
const VP8X_EXIF: u8 = 0x08;
const VP8X_XMP: u8 = 0x04;

/// Optional chunks to add to a WebP file.
#[derive(Clone, Copy, Default)]
pub(crate) struct Extras<'a> {
    pub icc: Option<&'a [u8]>,
    /// Bare TIFF block.
    pub exif: Option<&'a [u8]>,
    pub xmp: Option<&'a [u8]>,
}

impl Extras<'_> {
    fn is_empty(&self) -> bool {
        self.icc.is_none() && self.exif.is_none() && self.xmp.is_none()
    }

    fn flags(&self) -> u8 {
        let flag = |present: bool, bit: u8| if present { bit } else { 0 };
        flag(self.icc.is_some(), VP8X_ICC)
            | flag(self.exif.is_some(), VP8X_EXIF)
            | flag(self.xmp.is_some(), VP8X_XMP)
    }
}

/// Adds `ICCP`, `EXIF` and `XMP ` chunks to a WebP file, converting a
/// simple-format file (`VP8 `/`VP8L` only) to the extended format first.
/// Returns `None` if the input is not a well-formed WebP or the result would
/// exceed RIFF limits.
pub(crate) fn embed_metadata(
    webp: &[u8],
    extras: Extras<'_>,
    width: u32,
    height: u32,
    alpha: bool,
) -> Option<Vec<u8>> {
    if webp.get(0..4)? != b"RIFF" || webp.get(8..12)? != b"WEBP" {
        return None;
    }
    let mut chunks = Vec::new();
    let mut at = 12usize;
    while at < webp.len() {
        let kind = webp.get(at..at + 4)?;
        let size = usize::try_from(u32::from_le_bytes(
            webp.get(at + 4..at + 8)?.try_into().ok()?,
        ))
        .ok()?;
        let data_end = at.checked_add(8)?.checked_add(size)?;
        chunks.push((kind, webp.get(at + 8..data_end)?));
        // Chunks are padded to an even length.
        at = data_end.checked_add(size & 1)?;
    }

    let mut body = b"WEBP".to_vec();
    let mut push_chunk = |kind: &[u8], data: &[u8]| -> Option<()> {
        body.extend_from_slice(kind);
        body.extend_from_slice(&u32::try_from(data.len()).ok()?.to_le_bytes());
        body.extend_from_slice(data);
        if data.len() % 2 == 1 {
            body.push(0);
        }
        Some(())
    };

    let rest = match chunks.first() {
        Some((kind, data)) if *kind == b"VP8X" => {
            let mut header = data.to_vec();
            *header.first_mut()? |= extras.flags();
            push_chunk(b"VP8X", &header)?;
            &chunks[1..]
        }
        Some(_) => {
            // Build an extended header: flags, 3 reserved bytes, 24-bit
            // canvas width-1 and height-1.
            let alpha_flag = if alpha { VP8X_ALPHA } else { 0 };
            let mut header = vec![extras.flags() | alpha_flag, 0, 0, 0];
            header.extend_from_slice(&width.checked_sub(1)?.to_le_bytes()[..3]);
            header.extend_from_slice(&height.checked_sub(1)?.to_le_bytes()[..3]);
            push_chunk(b"VP8X", &header)?;
            &chunks[..]
        }
        None => return None,
    };
    // Required order: VP8X, ICCP, image data, EXIF, XMP.
    if let Some(profile) = extras.icc {
        push_chunk(b"ICCP", profile)?;
    }
    for (kind, data) in rest {
        push_chunk(kind, data)?;
    }
    if let Some(exif) = extras.exif {
        push_chunk(b"EXIF", exif)?;
    }
    if let Some(xmp) = extras.xmp {
        push_chunk(b"XMP ", xmp)?;
    }

    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&u32::try_from(body.len()).ok()?.to_le_bytes());
    out.extend(body);
    Some(out)
}

fn verify(
    bytes: &[u8],
    expected: &image::RgbaImage,
    lossless: bool,
    alpha: bool,
) -> Result<(), EncodeError> {
    let fail = EncodeError::VerificationFailed(F);
    let reopened = decode(bytes, &ImageLimits::default()).map_err(|_| fail.clone())?;
    if reopened.info.format != F
        || (reopened.info.width, reopened.info.height) != expected.dimensions()
    {
        return Err(fail);
    }
    let actual = reopened.pixels.to_rgba8();
    if lossless {
        // Compare RGBA exactly. Where the source had no transparency the
        // alpha channel was dropped, which decodes back as fully opaque.
        if actual != *expected {
            return Err(fail);
        }
        return Ok(());
    }
    if alpha && !has_transparency(&reopened.pixels) {
        return Err(fail);
    }
    let total: u64 = actual
        .as_raw()
        .iter()
        .zip(expected.as_raw())
        .map(|(a, b)| u64::from(a.abs_diff(*b)))
        .sum();
    // Coarse garbage check, not a quality metric.
    if total / expected.as_raw().len().max(1) as u64 > 48 {
        return Err(fail);
    }
    Ok(())
}

#[cfg(test)]
mod tests;

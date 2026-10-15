//! Decoder tests using real, fully valid files.
//!
//! Fixtures are generated in memory by the `image` crate's own encoders from
//! synthetic pixel patterns (gradients and a transparent quadrant), so they
//! are reproducible and redistributable (see `fixtures/README.md`).

use std::io::Cursor;

use image::{ImageBuffer, Rgb, Rgba};

use super::*;

/// RGBA pattern: a colour gradient whose top-left quadrant is fully
/// transparent and whose right column is half transparent.
fn rgba_pattern(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(width, height, |x, y| {
        let alpha = if x < width / 2 && y < height / 2 {
            0
        } else if x == width - 1 {
            128
        } else {
            255
        };
        Rgba([
            (x * 255 / width) as u8,
            (y * 255 / height) as u8,
            200,
            alpha,
        ])
    }))
}

fn rgb_gradient(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(width, height, |x, y| {
        Rgb([(x * 255 / width) as u8, (y * 255 / height) as u8, 128])
    }))
}

fn encode(image: &DynamicImage, format: image::ImageFormat) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, format)
        .unwrap_or_else(|error| unreachable!("test fixture failed to encode: {error}"));
    out.into_inner()
}

fn decode_ok(bytes: &[u8]) -> DecodedImage {
    decode(bytes, &ImageLimits::default())
        .unwrap_or_else(|error| unreachable!("expected successful decode, got {error:?}"))
}

#[test]
fn png_round_trip_is_pixel_exact_including_alpha() {
    let original = rgba_pattern(40, 30);
    let decoded = decode_ok(&encode(&original, image::ImageFormat::Png));
    assert_eq!(decoded.info.format, ImageFormat::Png);
    assert_eq!((decoded.pixels.width(), decoded.pixels.height()), (40, 30));
    assert_eq!(decoded.pixels.to_rgba8(), original.to_rgba8());
    assert_eq!(decoded.orientation, Orientation::NoTransforms);
}

#[test]
fn sixteen_bit_png_keeps_its_bit_depth() {
    let deep = DynamicImage::ImageRgb16(ImageBuffer::from_fn(8, 8, |x, y| {
        Rgb([(x * 8000) as u16, (y * 8000) as u16, 65_535])
    }));
    let decoded = decode_ok(&encode(&deep, image::ImageFormat::Png));
    assert_eq!(decoded.pixels.color(), image::ColorType::Rgb16);
    assert_eq!(decoded.pixels.to_rgb16(), deep.to_rgb16());
}

#[test]
fn webp_lossless_round_trip_is_pixel_exact_including_alpha() {
    let original = rgba_pattern(17, 9);
    let decoded = decode_ok(&encode(&original, image::ImageFormat::WebP));
    assert_eq!(decoded.info.format, ImageFormat::WebP);
    assert_eq!(decoded.pixels.to_rgba8(), original.to_rgba8());
}

#[test]
fn jpeg_round_trip_keeps_size_and_is_close() {
    let original = rgb_gradient(64, 48);
    let decoded = decode_ok(&encode(&original, image::ImageFormat::Jpeg));
    assert_eq!(decoded.info.format, ImageFormat::Jpeg);
    assert_eq!((decoded.pixels.width(), decoded.pixels.height()), (64, 48));
    // JPEG is lossy: compare mean absolute error instead of exact pixels.
    let a = original.to_rgb8();
    let b = decoded.pixels.to_rgb8();
    let total: u64 = a
        .as_raw()
        .iter()
        .zip(b.as_raw())
        .map(|(x, y)| u64::from(x.abs_diff(*y)))
        .sum();
    let mean = total as f64 / a.as_raw().len() as f64;
    assert!(
        mean < 4.0,
        "mean absolute error {mean} too high for a smooth gradient"
    );
}

#[test]
fn avif_is_reported_as_not_yet_supported() {
    // A header-valid AVIF is refused cleanly rather than half-decoded.
    let mut avif = Vec::new();
    let boxed = |kind: &[u8; 4], payload: &[u8]| {
        let mut out = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
        out.extend_from_slice(kind);
        out.extend_from_slice(payload);
        out
    };
    let ispe = boxed(b"ispe", &[0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 8]);
    let ipco = boxed(b"ipco", &ispe);
    let iprp = boxed(b"iprp", &ipco);
    let mut meta = vec![0, 0, 0, 0];
    meta.extend(iprp);
    avif.extend(boxed(b"ftyp", b"avif\0\0\0\0mif1avif"));
    avif.extend(boxed(b"meta", &meta));
    assert_eq!(
        decode(&avif, &ImageLimits::default()).map(|d| d.info),
        Err(DecodeError::FormatNotYetSupported(ImageFormat::Avif))
    );
}

#[test]
fn preflight_failures_pass_through_with_their_codes() {
    let result = decode(b"GIF89a\x01\x00\x01\x00", &ImageLimits::default());
    assert_eq!(
        result.map(|d| d.info).map_err(|e| e.code()),
        Err("unsupported-format")
    );
}

#[test]
fn damaged_pixel_data_fails_without_panicking() {
    for format in [
        image::ImageFormat::Png,
        image::ImageFormat::Jpeg,
        image::ImageFormat::WebP,
    ] {
        let source = if format == image::ImageFormat::Jpeg {
            rgb_gradient(32, 32)
        } else {
            rgba_pattern(32, 32)
        };
        let bytes = encode(&source, format);
        // Every truncation point, and a copy with the second half zeroed.
        for end in (0..bytes.len()).step_by(7) {
            let _ = decode(&bytes[..end], &ImageLimits::default());
        }
        let mut zeroed = bytes.clone();
        let half = zeroed.len() / 2;
        zeroed[half..].fill(0);
        let _ = decode(&zeroed, &ImageLimits::default());
    }
}

#[test]
fn truncated_png_is_reported_as_corrupt() {
    let bytes = encode(&rgba_pattern(32, 32), image::ImageFormat::Png);
    let cut = &bytes[..bytes.len() - 20]; // header intact, image data cut
    assert_eq!(
        decode(cut, &ImageLimits::default()).map(|d| d.info),
        Err(DecodeError::Corrupt(ImageFormat::Png))
    );
}

#[test]
fn header_that_lies_about_size_is_not_trusted() {
    // Valid 8x8 PNG whose IHDR is edited to claim 9x8. The PNG decoder checks
    // chunk CRCs, so this must fail as corrupt rather than over-read.
    let mut bytes = encode(&rgba_pattern(8, 8), image::ImageFormat::Png);
    bytes[16..20].copy_from_slice(&9u32.to_be_bytes());
    let result = decode(&bytes, &ImageLimits::default()).map(|d| d.info);
    assert!(
        matches!(
            result,
            Err(DecodeError::Corrupt(_) | DecodeError::DimensionMismatch { .. })
        ),
        "got {result:?}"
    );
}

#[test]
fn caller_limits_are_enforced_before_decoding() {
    let bytes = encode(&rgba_pattern(100, 100), image::ImageFormat::Png);
    let tight = ImageLimits {
        max_side: 99,
        max_pixels: 1 << 20,
    };
    assert_eq!(
        decode(&bytes, &tight).map(|d| d.info).map_err(|e| e.code()),
        Err("too-many-pixels")
    );
}

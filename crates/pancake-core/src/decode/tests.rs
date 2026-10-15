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

/// Encodes `image` as AVIF with ravif (fast speed setting, tests only).
fn encode_avif(image: &DynamicImage, depth: ravif::BitDepth) -> Vec<u8> {
    let rgba = image.to_rgba8();
    let pixels: Vec<ravif::RGBA8> = rgba
        .pixels()
        .map(|p| ravif::RGBA8::new(p[0], p[1], p[2], p[3]))
        .collect();
    let (width, height) = (rgba.width() as usize, rgba.height() as usize);
    ravif::Encoder::new()
        .with_quality(90.0)
        .with_alpha_quality(90.0)
        .with_speed(10)
        .with_bit_depth(depth)
        .with_num_threads(Some(1))
        .encode_rgba(ravif::Img::new(&pixels[..], width, height))
        .unwrap_or_else(|error| unreachable!("test fixture failed to encode: {error}"))
        .avif_file
}

fn mean_abs_error_rgba8(a: &DynamicImage, b: &DynamicImage) -> f64 {
    let (a, b) = (a.to_rgba8(), b.to_rgba8());
    let total: u64 = a
        .as_raw()
        .iter()
        .zip(b.as_raw())
        .map(|(x, y)| u64::from(x.abs_diff(*y)))
        .sum();
    total as f64 / a.as_raw().len() as f64
}

#[test]
fn avif_eight_bit_with_alpha_decodes_close_to_source() {
    let original = rgba_pattern(48, 32);
    let bytes = encode_avif(&original, ravif::BitDepth::Eight);
    // The header inspector must agree with a real encoder's output.
    assert_eq!(
        crate::format::inspect(&bytes).map(|i| (i.format, i.width, i.height)),
        Ok((ImageFormat::Avif, 48, 32))
    );
    let decoded = decode_ok(&bytes);
    assert_eq!((decoded.info.width, decoded.info.height), (48, 32));
    assert!(decoded.pixels.color().has_alpha());
    assert_eq!(decoded.info.alpha, crate::format::AlphaHint::Present);
    let error = mean_abs_error_rgba8(&original, &decoded.pixels);
    assert!(error < 6.0, "mean absolute error {error}");
    // The fully transparent quadrant must stay fully transparent.
    let rgba = decoded.pixels.to_rgba8();
    assert!(
        rgba.get_pixel(0, 0)[3] < 8,
        "alpha {}",
        rgba.get_pixel(0, 0)[3]
    );
}

#[test]
fn ten_bit_avif_is_full_range_sixteen_bit_not_dark() {
    // An opaque mid-grey: if 10-bit samples were not scaled to 16 bits the
    // decoded image would be about 64x too dark.
    let grey =
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(16, 16, Rgba([128, 128, 128, 255])));
    let decoded = decode_ok(&encode_avif(&grey, ravif::BitDepth::Ten));
    assert!(matches!(
        decoded.pixels.color(),
        image::ColorType::Rgb16 | image::ColorType::Rgba16
    ));
    let centre = decoded.pixels.to_rgb16().get_pixel(8, 8)[0];
    assert!(
        (30_000..36_000).contains(&centre),
        "16-bit value {centre} is not mid-grey (about 32,896)"
    );
}

#[test]
fn avif_frame_larger_than_container_is_refused() {
    // Real 32x32 AVIF whose container size (ispe) is edited down to 8x8.
    // The AV1 stream still declares 32x32, so decoding must be refused
    // before the AV1 decoder allocates for the larger frame.
    let mut bytes = encode_avif(&rgba_pattern(32, 32), ravif::BitDepth::Eight);
    let mut edited = 0;
    let mut at = 0;
    while let Some(found) = bytes[at..].windows(4).position(|w| w == b"ispe") {
        let size_at = at + found + 4 + 4; // after "ispe" and version/flags
        bytes[size_at..size_at + 4].copy_from_slice(&8u32.to_be_bytes());
        bytes[size_at + 4..size_at + 8].copy_from_slice(&8u32.to_be_bytes());
        edited += 1;
        at = size_at;
    }
    assert!(edited >= 1, "fixture has no ispe box");
    let result = decode(&bytes, &ImageLimits::default()).map(|d| d.info);
    assert!(
        matches!(result, Err(DecodeError::DimensionMismatch { .. })),
        "got {result:?}"
    );
}

#[test]
fn damaged_avif_fails_without_panicking() {
    let bytes = encode_avif(&rgba_pattern(24, 24), ravif::BitDepth::Eight);
    for end in (0..bytes.len()).step_by(5) {
        let _ = decode(&bytes[..end], &ImageLimits::default());
    }
    let mut zeroed = bytes.clone();
    let half = zeroed.len() / 2;
    zeroed[half..].fill(0);
    let _ = decode(&zeroed, &ImageLimits::default());
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

#[test]
fn preview_is_small_upright_and_reports_full_size() {
    let mut bytes = encode(&rgb_gradient(1200, 600), image::ImageFormat::Png);
    // Without orientation: 1200 x 600, scaled to fit 300.
    let (png, width, height) = preview_png(&bytes, &ImageLimits::default(), 300)
        .unwrap_or_else(|error| unreachable!("{error:?}"));
    assert_eq!((width, height), (1200, 600));
    let thumb = decode_ok(&png);
    assert_eq!((thumb.info.width, thumb.info.height), (300, 150));
    // Small images are not upscaled.
    bytes = encode(&rgb_gradient(40, 20), image::ImageFormat::Png);
    let (png, _, _) = preview_png(&bytes, &ImageLimits::default(), 300)
        .unwrap_or_else(|error| unreachable!("{error:?}"));
    assert_eq!(decode_ok(&png).info.width, 40);
}

#[test]
fn display_pixels_apply_orientation() {
    let mut decoded = decode_ok(&encode(&rgb_gradient(40, 20), image::ImageFormat::Png));
    decoded.orientation = Orientation::Rotate90;
    let shown = display_pixels(&decoded);
    assert_eq!((shown.width(), shown.height()), (20, 40));
}

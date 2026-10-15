//! WebP encoder tests. Fixtures are synthetic images built in memory.

use std::io::Cursor;

use image::metadata::Orientation;
use image::{DynamicImage, ImageBuffer, Rgb, Rgba};

use super::*;

fn decoded_from(image: &DynamicImage) -> DecodedImage {
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap_or_else(|error| unreachable!("fixture encode failed: {error}"));
    decode(&bytes.into_inner(), &ImageLimits::default())
        .unwrap_or_else(|error| unreachable!("fixture decode failed: {error:?}"))
}

fn photo(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(width, height, |x, y| {
        Rgb([(x * 255 / width) as u8, (y * 255 / height) as u8, 90])
    }))
}

/// Transparent left half with hidden non-zero RGB; opaque right half.
fn cutout(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(width, height, |x, y| {
        if x < width / 2 {
            Rgba([77, 66, 55, 0])
        } else {
            Rgba([(y * 255 / height) as u8, 120, 200, 255])
        }
    }))
}

fn encode_ok(decoded: &DecodedImage, options: WebpOptions) -> Encoded {
    encode(decoded, options).unwrap_or_else(|error| unreachable!("{error:?}"))
}

fn reopen(bytes: &[u8]) -> DecodedImage {
    decode(bytes, &ImageLimits::default()).unwrap_or_else(|error| unreachable!("{error:?}"))
}

const LOSSLESS: WebpOptions = WebpOptions {
    lossless: true,
    quality: 75,
    method: 4,
};

#[test]
fn lossless_is_pixel_identical_including_hidden_rgb() {
    let source = cutout(40, 24);
    let output = encode_ok(&decoded_from(&source), LOSSLESS);
    assert_eq!(reopen(&output.bytes).pixels.to_rgba8(), source.to_rgba8());
}

/// Gradient plus deterministic grain: closer to a real photo than a pure
/// gradient, which libwebp's lossless predictor compresses almost to nothing
/// (lossless can then beat lossy, a real effect the M2 "avoid larger output"
/// policy exists for).
fn textured_photo(width: u32, height: u32) -> DynamicImage {
    let mut state: u32 = 0xC0FF_EE11;
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(width, height, |x, y| {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let grain = (state % 24) as u8;
        Rgb([
            ((x * 200 / width) as u8).saturating_add(grain),
            ((y * 200 / height) as u8).saturating_add(grain / 2),
            90u8.saturating_add(grain),
        ])
    }))
}

#[test]
fn lossy_photo_decodes_and_is_smaller_than_lossless() {
    let source = decoded_from(&textured_photo(128, 96));
    let lossy = encode_ok(&source, WebpOptions::default());
    let lossless = encode_ok(&source, LOSSLESS);
    let back = reopen(&lossy.bytes);
    assert_eq!((back.info.width, back.info.height), (128, 96));
    assert!(!has_transparency(&back.pixels));
    assert!(lossy.bytes.len() < lossless.bytes.len());
}

#[test]
fn lossy_keeps_transparency() {
    let output = encode_ok(&decoded_from(&cutout(32, 32)), WebpOptions::default());
    assert!(!output.adjustments.transparency_flattened);
    let back = reopen(&output.bytes).pixels.to_rgba8();
    assert!(
        back.get_pixel(2, 2)[3] < 8,
        "alpha {}",
        back.get_pixel(2, 2)[3]
    );
    assert!(back.get_pixel(30, 2)[3] > 247);
}

#[test]
fn icc_profile_survives_every_webp_layout() {
    // Odd length forces a padding byte after the ICCP chunk.
    let profile: Vec<u8> = (0..1001u32).map(|i| (i % 253) as u8).collect();
    let cases = [
        (photo(24, 16), WebpOptions::default()), // simple lossy (VP8)
        (photo(24, 16), LOSSLESS),               // simple lossless (VP8L)
        (cutout(24, 16), WebpOptions::default()), // extended lossy + ALPH
    ];
    for (image, options) in cases {
        let mut source = decoded_from(&image);
        source.icc_profile = Some(profile.clone());
        let output = encode_ok(&source, options);
        let back = reopen(&output.bytes);
        assert_eq!(back.icc_profile.as_ref(), Some(&profile), "{options:?}");
        // The RIFF size field must match the actual length.
        let riff_size = u32::from_le_bytes([
            output.bytes[4],
            output.bytes[5],
            output.bytes[6],
            output.bytes[7],
        ]);
        assert_eq!(riff_size as usize + 8, output.bytes.len());
    }
}

#[test]
fn embed_metadata_rejects_non_webp_input() {
    let extras = Extras {
        icc: Some(b"icc"),
        ..Extras::default()
    };
    assert_eq!(
        embed_metadata(b"not a webp file", extras, 1, 1, false),
        None
    );
}

#[test]
fn orientation_and_precision_are_applied_and_reported() {
    let deep = DynamicImage::ImageRgb16(ImageBuffer::from_fn(30, 10, |x, _| {
        Rgb([(x * 2000) as u16, 30_000, 60_000])
    }));
    let mut source = decoded_from(&deep);
    source.orientation = Orientation::Rotate270;
    let output = encode_ok(&source, WebpOptions::default());
    assert!(output.adjustments.orientation_applied);
    assert!(output.adjustments.precision_reduced);
    let back = reopen(&output.bytes);
    assert_eq!((back.info.width, back.info.height), (10, 30));
}

#[test]
fn invalid_options_are_rejected() {
    let source = decoded_from(&photo(8, 8));
    for options in [
        WebpOptions {
            quality: 101,
            ..WebpOptions::default()
        },
        WebpOptions {
            method: 7,
            ..WebpOptions::default()
        },
    ] {
        assert!(matches!(
            encode(&source, options),
            Err(EncodeError::InvalidOptions(_))
        ));
    }
}

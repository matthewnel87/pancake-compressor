//! PNG encoder tests. Fixtures are synthetic images built in memory.

use std::io::Cursor;

use image::{DynamicImage, ImageBuffer, Rgb, Rgba};

use super::*;

fn png_bytes(image: &DynamicImage) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, image::ImageFormat::Png)
        .unwrap_or_else(|error| unreachable!("fixture encode failed: {error}"));
    out.into_inner()
}

fn decoded(bytes: &[u8]) -> DecodedImage {
    decode(bytes, &ImageLimits::default())
        .unwrap_or_else(|error| unreachable!("fixture decode failed: {error:?}"))
}

/// Flat-colour "logo": four colours, a fully transparent quadrant whose
/// hidden RGB is non-zero (oxipng's optimize_alpha would rewrite it).
fn logo(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(width, height, |x, y| {
        match (x < width / 2, y < height / 2) {
            (true, true) => Rgba([200, 30, 30, 0]),
            (false, true) => Rgba([30, 120, 200, 255]),
            (true, false) => Rgba([250, 210, 120, 255]),
            (false, false) => Rgba([40, 40, 40, 128]),
        }
    }))
}

/// Deterministic noise that no 256-colour palette can represent well.
fn noise(width: u32, height: u32) -> DynamicImage {
    let mut state: u32 = 0x1234_5678;
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(width, height, |_, _| {
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state & 0xFF) as u8
        };
        Rgb([next(), next(), next()])
    }))
}

fn colour_type(png: &[u8]) -> u8 {
    png[25] // IHDR colour type byte
}

#[test]
fn lossless_output_is_pixel_identical_including_hidden_rgb() {
    let source = png_bytes(&logo(64, 64));
    let before = decoded(&source);
    let output = optimise_lossless(&source, &before, LosslessOptions::default())
        .unwrap_or_else(|error| unreachable!("{error:?}"));
    let after = decoded(&output);
    // Compare raw RGBA including the RGB of fully transparent pixels.
    assert_eq!(after.pixels.to_rgba16(), before.pixels.to_rgba16());
    assert!(
        output.len() <= source.len(),
        "{} > {}",
        output.len(),
        source.len()
    );
}

#[test]
fn lossless_keeps_sixteen_bit_precision() {
    let deep = DynamicImage::ImageRgb16(ImageBuffer::from_fn(16, 16, |x, y| {
        Rgb([(x * 4000 + 1) as u16, (y * 4000 + 3) as u16, 12_345])
    }));
    let source = png_bytes(&deep);
    let before = decoded(&source);
    let output = optimise_lossless(&source, &before, LosslessOptions { effort: 4 })
        .unwrap_or_else(|error| unreachable!("{error:?}"));
    assert_eq!(decoded(&output).pixels.to_rgb16(), deep.to_rgb16());
}

#[test]
fn lossless_keeps_text_metadata() {
    // Write a PNG with a tEXt chunk directly via the png crate.
    let image = logo(8, 8).to_rgba8();
    let mut source = Vec::new();
    {
        let mut encoder = ::png::Encoder::new(&mut source, 8, 8);
        encoder.set_color(::png::ColorType::Rgba);
        encoder.set_depth(::png::BitDepth::Eight);
        encoder
            .add_text_chunk("Author".into(), "Synthetic fixture".into())
            .unwrap_or_else(|error| unreachable!("{error}"));
        let mut writer = encoder
            .write_header()
            .unwrap_or_else(|error| unreachable!("{error}"));
        writer
            .write_image_data(image.as_raw())
            .unwrap_or_else(|error| unreachable!("{error}"));
    }
    let output = optimise_lossless(&source, &decoded(&source), LosslessOptions::default())
        .unwrap_or_else(|error| unreachable!("{error:?}"));
    let found = output
        .windows(b"Synthetic fixture".len())
        .any(|w| w == b"Synthetic fixture");
    assert!(found, "tEXt metadata was dropped");
}

#[test]
fn lossy_logo_becomes_small_indexed_png_and_keeps_transparency() {
    let source = logo(128, 128);
    let input = decoded(&png_bytes(&source));
    let output = quantise_lossy(&input, LossyOptions::default())
        .unwrap_or_else(|error| unreachable!("{error:?}"))
        .bytes;
    assert_eq!(colour_type(&output), 3, "expected an indexed PNG");
    let after = decoded(&output).pixels.to_rgba8();
    // Four flat colours fit a palette exactly, including the alpha values.
    assert_eq!(after.get_pixel(0, 0)[3], 0);
    assert_eq!(after.get_pixel(127, 127)[3], 128);
    assert_eq!(after.get_pixel(127, 0), &Rgba([30, 120, 200, 255]));
}

#[test]
fn lossy_refuses_rather_than_banding_when_quality_floor_is_unreachable() {
    let input = decoded(&png_bytes(&noise(64, 64)));
    let strict = LossyOptions {
        min_quality: 95,
        target_quality: 100,
        ..LossyOptions::default()
    };
    assert_eq!(
        quantise_lossy(&input, strict).map(|e| e.adjustments),
        Err(EncodeError::QualityNotReached(ImageFormat::Png))
    );
}

#[test]
fn lossy_reduces_sixteen_bit_sources_to_eight_bit_palette() {
    let deep = DynamicImage::ImageRgba16(ImageBuffer::from_fn(32, 32, |x, _| {
        let v = if x < 16 { 10_000 } else { 50_000 };
        Rgba([v, v, v, 65_535])
    }));
    let input = decoded(&png_bytes(&deep));
    let output = quantise_lossy(&input, LossyOptions::default())
        .unwrap_or_else(|error| unreachable!("{error:?}"))
        .bytes;
    assert_eq!(colour_type(&output), 3);
}

#[test]
fn lossy_carries_the_icc_profile() {
    let mut input = decoded(&png_bytes(&logo(16, 16)));
    // Content is opaque to PNG; a real profile is not needed to test carriage.
    let profile = b"synthetic-icc-profile-bytes".to_vec();
    input.icc_profile = Some(profile.clone());
    let output = quantise_lossy(&input, LossyOptions::default())
        .unwrap_or_else(|error| unreachable!("{error:?}"))
        .bytes;
    assert_eq!(decoded(&output).icc_profile, Some(profile));
}

#[test]
fn invalid_options_are_rejected_before_work() {
    let input = decoded(&png_bytes(&logo(8, 8)));
    let bad = [
        LossyOptions {
            min_quality: 90,
            target_quality: 80,
            ..LossyOptions::default()
        },
        LossyOptions {
            speed: 0,
            ..LossyOptions::default()
        },
        LossyOptions {
            dithering: 1.5,
            ..LossyOptions::default()
        },
    ];
    for options in bad {
        assert!(matches!(
            quantise_lossy(&input, options),
            Err(EncodeError::InvalidOptions(_))
        ));
    }
    let source = png_bytes(&logo(8, 8));
    assert!(matches!(
        optimise_lossless(&source, &input, LosslessOptions { effort: 7 }),
        Err(EncodeError::InvalidOptions(_))
    ));
}

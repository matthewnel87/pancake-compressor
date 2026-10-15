//! Smart Auto and preset tests. Fixtures are synthetic images built in memory.

use std::io::Cursor;

use image::{DynamicImage, ImageBuffer, Rgb, Rgba};

use super::*;
use crate::decode::decode;
use crate::limits::ImageLimits;

fn as_png(image: &DynamicImage) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, image::ImageFormat::Png)
        .unwrap_or_else(|error| unreachable!("{error}"));
    out.into_inner()
}

fn decoded(bytes: &[u8]) -> DecodedImage {
    decode(bytes, &ImageLimits::default()).unwrap_or_else(|error| unreachable!("{error:?}"))
}

/// Photo-like: gradient plus grain, thousands of colours, opaque.
fn photo() -> DynamicImage {
    let mut state: u32 = 0xABCD_1234;
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(160, 120, |x, y| {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let grain = (state % 20) as u8;
        Rgb([
            ((x * 200 / 160) as u8).saturating_add(grain),
            ((y * 200 / 120) as u8).saturating_add(grain),
            80u8.saturating_add(grain),
        ])
    }))
}

/// Graphic: four flat colours with a transparent area.
fn logo() -> DynamicImage {
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(96, 96, |x, y| {
        match (x < 48, y < 48) {
            (true, true) => Rgba([0, 0, 0, 0]),
            (false, true) => Rgba([220, 40, 40, 255]),
            (true, false) => Rgba([40, 40, 220, 255]),
            (false, false) => Rgba([250, 250, 250, 255]),
        }
    }))
}

fn run(source: &[u8], target: Target, preset: Preset, consent: Consent) -> Outcome {
    compress(source, &decoded(source), target, preset, consent)
        .unwrap_or_else(|error| unreachable!("{error:?}"))
}

#[test]
fn auto_picks_a_lossy_web_format_for_photos_and_is_deterministic() {
    let source = as_png(&photo());
    let first = run(&source, Target::Auto, Preset::Balanced, Consent::default());
    assert!(matches!(
        first.output_format,
        ImageFormat::WebP | ImageFormat::Jpeg
    ));
    assert_eq!(first.reason, Reason::Smallest);
    assert!(!first.kept_original);
    let second = run(&source, Target::Auto, Preset::Balanced, Consent::default());
    assert_eq!(
        first, second,
        "same input and settings must give the same result"
    );
}

#[test]
fn auto_never_offers_jpeg_for_transparent_graphics() {
    let source = as_png(&logo());
    let outcome = run(&source, Target::Auto, Preset::Balanced, Consent::default());
    assert!(matches!(
        outcome.output_format,
        ImageFormat::Png | ImageFormat::WebP
    ));
    assert!(!outcome.adjustments.transparency_flattened);
    let back = decoded(&outcome.bytes).pixels.to_rgba8();
    assert_eq!(back.get_pixel(5, 5)[3], 0, "transparency lost");
}

#[test]
fn keep_format_stays_in_the_source_format() {
    let source = as_png(&logo());
    let outcome = run(&source, Target::Keep, Preset::Balanced, Consent::default());
    assert_eq!(outcome.output_format, ImageFormat::Png);
}

#[test]
fn lossless_preset_keeps_jpeg_original_and_says_why() {
    let jpeg = crate::encode::jpeg::encode(&decoded(&as_png(&photo())), Default::default())
        .unwrap_or_else(|error| unreachable!("{error:?}"))
        .bytes;
    let outcome = run(&jpeg, Target::Keep, Preset::Lossless, Consent::default());
    assert!(outcome.kept_original);
    assert_eq!(outcome.reason, Reason::NoSuitableMode);
    assert_eq!(outcome.bytes, jpeg);
}

#[test]
fn never_writes_a_larger_file() {
    // A small, heavily compressed JPEG re-encoded at Best quality would grow.
    let small = crate::encode::jpeg::encode(
        &decoded(&as_png(&photo())),
        crate::encode::jpeg::JpegOptions {
            quality: 20,
            ..Default::default()
        },
    )
    .unwrap_or_else(|error| unreachable!("{error:?}"))
    .bytes;
    let outcome = run(
        &small,
        Target::Keep,
        Preset::BestQuality,
        Consent::default(),
    );
    assert!(outcome.kept_original);
    assert_eq!(outcome.reason, Reason::OriginalSmaller);
    assert_eq!(outcome.bytes, small);
}

#[test]
fn lossless_auto_is_pixel_identical() {
    let source = as_png(&photo());
    let outcome = run(&source, Target::Auto, Preset::Lossless, Consent::default());
    let original = decoded(&source).pixels.to_rgba8();
    let result = decoded(&outcome.bytes).pixels.to_rgba8();
    assert_eq!(original, result);
}

#[test]
fn avif_needs_consent_to_drop_a_colour_profile() {
    let source = as_png(&photo());
    let mut with_profile = decoded(&source);
    with_profile.icc_profile = Some(b"synthetic-icc".to_vec());
    let refused = compress(
        &source,
        &with_profile,
        Target::Avif,
        Preset::Balanced,
        Consent::default(),
    );
    assert_eq!(
        refused.map(|o| o.output_format),
        Err(EncodeError::ColourProfileNotSupported(ImageFormat::Avif))
    );
    let consent = Consent {
        allow_colour_profile_loss: true,
        ..Consent::default()
    };
    let allowed = compress(
        &source,
        &with_profile,
        Target::Avif,
        Preset::Smaller,
        consent,
    )
    .unwrap_or_else(|error| unreachable!("{error:?}"));
    assert!(allowed.adjustments.colour_profile_dropped);
}

#[test]
fn avif_is_only_tried_in_auto_when_enabled() {
    let source = as_png(&photo());
    let without = run(&source, Target::Auto, Preset::Smaller, Consent::default());
    assert_ne!(without.output_format, ImageFormat::Avif);
    let with = run(
        &source,
        Target::Auto,
        Preset::Smaller,
        Consent {
            try_avif_in_auto: true,
            ..Consent::default()
        },
    );
    // AVIF is usually smallest for photos; at minimum it must not be larger
    // than the run without it.
    assert!(with.bytes.len() <= without.bytes.len());
}

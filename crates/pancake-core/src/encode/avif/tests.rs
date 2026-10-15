//! AVIF encoder tests. Fixtures are synthetic images built in memory.
//! Speed 10 keeps AV1 encoding fast enough for unit tests.

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

const FAST: AvifOptions = AvifOptions {
    quality: 70.0,
    alpha_quality: 90.0,
    speed: 10,
    threads: 1,
    allow_colour_profile_loss: false,
};

fn encode_ok(decoded: &DecodedImage, options: AvifOptions) -> Encoded {
    encode(decoded, options).unwrap_or_else(|error| unreachable!("{error:?}"))
}

fn reopen(bytes: &[u8]) -> DecodedImage {
    decode(bytes, &ImageLimits::default()).unwrap_or_else(|error| unreachable!("{error:?}"))
}

#[test]
fn photo_encodes_and_lower_quality_is_smaller() {
    let source = decoded_from(&photo(96, 64));
    let high = encode_ok(
        &source,
        AvifOptions {
            quality: 95.0,
            ..FAST
        },
    );
    let low = encode_ok(
        &source,
        AvifOptions {
            quality: 30.0,
            ..FAST
        },
    );
    assert_eq!(high.adjustments, Adjustments::default());
    let back = reopen(&low.bytes);
    assert_eq!(back.info.format, ImageFormat::Avif);
    assert_eq!((back.info.width, back.info.height), (96, 64));
    assert!(
        low.bytes.len() < high.bytes.len(),
        "{} >= {}",
        low.bytes.len(),
        high.bytes.len()
    );
}

#[test]
fn transparency_is_kept() {
    let cutout = DynamicImage::ImageRgba8(ImageBuffer::from_fn(32, 32, |x, _| {
        if x < 16 {
            Rgba([77, 66, 55, 0])
        } else {
            Rgba([30, 120, 200, 255])
        }
    }));
    let output = encode_ok(&decoded_from(&cutout), FAST);
    let back = reopen(&output.bytes).pixels.to_rgba8();
    assert!(
        back.get_pixel(2, 2)[3] < 8,
        "alpha {}",
        back.get_pixel(2, 2)[3]
    );
    assert!(back.get_pixel(30, 2)[3] > 247);
}

#[test]
fn colour_profile_is_refused_unless_loss_is_allowed() {
    let mut source = decoded_from(&photo(16, 16));
    source.icc_profile = Some(b"synthetic-icc".to_vec());
    assert_eq!(
        encode(&source, FAST).map(|e| e.adjustments),
        Err(EncodeError::ColourProfileNotSupported(ImageFormat::Avif))
    );
    let allowed = encode_ok(
        &source,
        AvifOptions {
            allow_colour_profile_loss: true,
            ..FAST
        },
    );
    assert!(allowed.adjustments.colour_profile_dropped);
}

#[test]
fn orientation_is_applied() {
    let mut source = decoded_from(&photo(40, 20));
    source.orientation = Orientation::Rotate90;
    let output = encode_ok(&source, FAST);
    assert!(output.adjustments.orientation_applied);
    let back = reopen(&output.bytes);
    assert_eq!((back.info.width, back.info.height), (20, 40));
}

#[test]
fn invalid_options_are_rejected() {
    let source = decoded_from(&photo(8, 8));
    for options in [
        AvifOptions {
            quality: 0.0,
            ..FAST
        },
        AvifOptions {
            alpha_quality: 101.0,
            ..FAST
        },
        AvifOptions { speed: 0, ..FAST },
        AvifOptions { threads: 0, ..FAST },
    ] {
        assert!(matches!(
            encode(&source, options),
            Err(EncodeError::InvalidOptions(_))
        ));
    }
}

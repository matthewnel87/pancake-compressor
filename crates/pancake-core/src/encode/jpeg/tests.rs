//! JPEG encoder tests. Fixtures are synthetic images built in memory.

use std::io::Cursor;

use image::metadata::Orientation;
use image::{DynamicImage, ImageBuffer, Luma, Rgb, Rgba};

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

fn encode_ok(decoded: &DecodedImage, options: JpegOptions) -> Encoded {
    encode(decoded, options).unwrap_or_else(|error| unreachable!("{error:?}"))
}

fn reopen(bytes: &[u8]) -> DecodedImage {
    decode(bytes, &ImageLimits::default()).unwrap_or_else(|error| unreachable!("{error:?}"))
}

#[test]
fn photo_encodes_to_valid_jpeg_with_no_adjustments() {
    let output = encode_ok(&decoded_from(&photo(96, 64)), JpegOptions::default());
    assert_eq!(output.adjustments, Adjustments::default());
    let back = reopen(&output.bytes);
    assert_eq!(back.info.format, ImageFormat::Jpeg);
    assert_eq!((back.info.width, back.info.height), (96, 64));
}

#[test]
fn transparency_is_flattened_onto_the_matte_and_reported() {
    let logo = DynamicImage::ImageRgba8(ImageBuffer::from_fn(32, 32, |x, _| {
        if x < 16 {
            Rgba([10, 200, 10, 0]) // fully transparent, hidden green
        } else {
            Rgba([200, 10, 10, 255])
        }
    }));
    let source = decoded_from(&logo);

    let white = encode_ok(&source, JpegOptions::default());
    assert!(white.adjustments.transparency_flattened);
    let pixel = reopen(&white.bytes).pixels.to_rgb8().get_pixel(4, 16).0;
    assert!(
        pixel.iter().all(|&c| c > 240),
        "expected white, got {pixel:?}"
    );

    let black = encode_ok(
        &source,
        JpegOptions {
            matte: [0, 0, 0],
            ..JpegOptions::default()
        },
    );
    let pixel = reopen(&black.bytes).pixels.to_rgb8().get_pixel(4, 16).0;
    assert!(
        pixel.iter().all(|&c| c < 16),
        "expected black, got {pixel:?}"
    );
}

#[test]
fn exif_orientation_is_applied_and_no_tag_is_written() {
    let mut source = decoded_from(&photo(40, 20));
    source.orientation = Orientation::Rotate90;
    let output = encode_ok(&source, JpegOptions::default());
    assert!(output.adjustments.orientation_applied);
    let back = reopen(&output.bytes);
    assert_eq!((back.info.width, back.info.height), (20, 40));
    assert_eq!(back.orientation, Orientation::NoTransforms);
    let has_exif = output.bytes.windows(6).any(|w| w == b"Exif\0\0");
    assert!(
        !has_exif,
        "an EXIF block could re-rotate the upright pixels"
    );
}

#[test]
fn greyscale_stays_greyscale() {
    let grey = DynamicImage::ImageLuma8(ImageBuffer::from_fn(32, 32, |x, _| Luma([(x * 8) as u8])));
    let output = encode_ok(&decoded_from(&grey), JpegOptions::default());
    let back = reopen(&output.bytes);
    assert_eq!(back.pixels.color(), image::ColorType::L8);
}

#[test]
fn sixteen_bit_source_is_reported_as_precision_reduced() {
    let deep = DynamicImage::ImageRgb16(ImageBuffer::from_fn(16, 16, |x, _| {
        Rgb([(x * 4000) as u16, 30_000, 60_000])
    }));
    let output = encode_ok(&decoded_from(&deep), JpegOptions::default());
    assert!(output.adjustments.precision_reduced);
}

#[test]
fn icc_profile_is_carried_including_multi_segment_profiles() {
    let mut source = decoded_from(&photo(16, 16));
    // Larger than one APP2 segment, so it must be split and reassembled.
    let profile: Vec<u8> = (0..140_000u32).map(|i| (i % 251) as u8).collect();
    source.icc_profile = Some(profile.clone());
    let output = encode_ok(&source, JpegOptions::default());
    assert_eq!(reopen(&output.bytes).icc_profile, Some(profile));
}

#[test]
fn icc_segments_follow_the_icc_layout() {
    let segments = icc_segments(&vec![7u8; 65_519 * 2 + 10]);
    assert_eq!(segments.len(), 3);
    for (index, segment) in segments.iter().enumerate() {
        assert!(segment.starts_with(b"ICC_PROFILE\0"));
        assert_eq!(segment[12], index as u8 + 1);
        assert_eq!(segment[13], 3);
    }
    assert_eq!(segments[2].len(), 14 + 10);
}

#[test]
fn lower_quality_gives_smaller_files() {
    let source = decoded_from(&photo(128, 128));
    let high = encode_ok(
        &source,
        JpegOptions {
            quality: 95,
            subsampling: Subsampling::Full,
            ..JpegOptions::default()
        },
    );
    let low = encode_ok(
        &source,
        JpegOptions {
            quality: 40,
            ..JpegOptions::default()
        },
    );
    assert!(
        low.bytes.len() < high.bytes.len(),
        "{} >= {}",
        low.bytes.len(),
        high.bytes.len()
    );
}

#[test]
fn quality_out_of_range_is_rejected() {
    let source = decoded_from(&photo(8, 8));
    for quality in [0, 101] {
        assert!(matches!(
            encode(
                &source,
                JpegOptions {
                    quality,
                    ..JpegOptions::default()
                }
            ),
            Err(EncodeError::InvalidOptions(_))
        ));
    }
}

#[test]
fn oversized_icc_profile_is_refused_not_dropped() {
    let mut source = decoded_from(&photo(8, 8));
    source.icc_profile = Some(vec![0u8; 65_519 * 256]);
    assert_eq!(
        encode(&source, JpegOptions::default()).map(|e| e.adjustments),
        Err(EncodeError::Failed(ImageFormat::Jpeg))
    );
}

//! EXIF/XMP carriage through every converting encoder (PNK-013, ADR-034).
//!
//! The source is a synthetic "camera" image: pixels stored sideways with an
//! EXIF orientation of 6 (rotate 90° clockwise to view), a camera make, and
//! an XMP packet. Every output must show the picture upright, keep the EXIF
//! (now saying "upright" so nothing rotates it twice) and keep the XMP where
//! the format can, reporting anything it cannot keep.

#![allow(clippy::panic)]

use std::io::Cursor;

use image::metadata::Orientation;
use image::{DynamicImage, ImageBuffer, Rgb};
use pancake_core::decode::{DecodedImage, decode};
use pancake_core::encode::{EncodeError, Encoded, avif, jpeg, png, webp};
use pancake_core::limits::ImageLimits;

const MAKE: &[u8] = b"PancakeTestCam\0";
const XMP: &str = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:rights>Synthetic fixture</dc:rights></rdf:Description></rdf:RDF></x:xmpmeta>"#;

/// Little-endian TIFF with IFD0 = [Make (ASCII), Orientation = 6].
fn camera_exif() -> Vec<u8> {
    let mut tiff = b"II*\0".to_vec();
    tiff.extend(8u32.to_le_bytes());
    tiff.extend(2u16.to_le_bytes());
    let strings_at: u32 = 8 + 2 + 2 * 12 + 4;
    // Make: tag 0x010F, type ASCII (2), count, offset to the string.
    tiff.extend(0x010Fu16.to_le_bytes());
    tiff.extend(2u16.to_le_bytes());
    tiff.extend((MAKE.len() as u32).to_le_bytes());
    tiff.extend(strings_at.to_le_bytes());
    // Orientation: tag 0x0112, SHORT (3), count 1, value 6.
    tiff.extend(0x0112u16.to_le_bytes());
    tiff.extend(3u16.to_le_bytes());
    tiff.extend(1u32.to_le_bytes());
    tiff.extend([6, 0, 0, 0]);
    tiff.extend(0u32.to_le_bytes()); // no next IFD
    tiff.extend(MAKE);
    let mut exif = b"Exif\0\0".to_vec();
    exif.extend(tiff);
    exif
}

/// 40 x 20 pixels stored sideways, as a camera would.
fn camera_source() -> DecodedImage {
    let image = DynamicImage::ImageRgb8(ImageBuffer::from_fn(40, 20, |x, y| {
        Rgb([(x * 6) as u8, (y * 12) as u8, 100])
    }));
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap_or_else(|error| panic!("{error}"));
    let mut source = decode(&bytes.into_inner(), &ImageLimits::default())
        .unwrap_or_else(|error| panic!("{error:?}"));
    source.orientation = Orientation::Rotate90;
    source.exif = Some(camera_exif());
    source.xmp = Some(XMP.as_bytes().to_vec());
    source
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn check(name: &str, output: &Encoded, xmp_expected: bool) {
    let back = decode(&output.bytes, &ImageLimits::default())
        .unwrap_or_else(|error| panic!("{name}: {error:?}"));
    assert!(output.adjustments.orientation_applied, "{name}");
    assert_eq!(
        (back.info.width, back.info.height),
        (20, 40),
        "{name}: not upright"
    );
    assert!(
        contains(&output.bytes, b"PancakeTestCam"),
        "{name}: EXIF lost"
    );
    // If the decoder can read this format's EXIF, it must now say upright.
    if back.exif.is_some() {
        assert_eq!(
            back.orientation,
            Orientation::NoTransforms,
            "{name}: would rotate twice"
        );
    }
    assert_eq!(
        contains(&output.bytes, b"Synthetic fixture"),
        xmp_expected,
        "{name}: XMP"
    );
    assert_eq!(
        output.adjustments.metadata_dropped, !xmp_expected,
        "{name}: drop report"
    );
}

#[test]
fn exif_and_xmp_are_carried_and_orientation_is_reset() {
    let source = camera_source();
    let ok = |name: &str, result: Result<Encoded, EncodeError>| -> Encoded {
        result.unwrap_or_else(|error| panic!("{name}: {error:?}"))
    };
    check(
        "jpeg",
        &ok("jpeg", jpeg::encode(&source, jpeg::JpegOptions::default())),
        true,
    );
    check(
        "webp lossy",
        &ok("webp", webp::encode(&source, webp::WebpOptions::default())),
        true,
    );
    check(
        "webp lossless",
        &ok(
            "webp lossless",
            webp::encode(
                &source,
                webp::WebpOptions {
                    lossless: true,
                    ..webp::WebpOptions::default()
                },
            ),
        ),
        true,
    );
    check(
        "png palette",
        &ok(
            "png",
            png::quantise_lossy(&source, png::LossyOptions::default()),
        ),
        true,
    );
    // ravif stores EXIF but not XMP: XMP must be reported as dropped.
    let fast = avif::AvifOptions {
        speed: 10,
        ..avif::AvifOptions::default()
    };
    check("avif", &ok("avif", avif::encode(&source, fast)), false);
}

#[test]
fn unreadable_exif_is_dropped_and_reported_not_copied() {
    let mut source = camera_source();
    source.exif = Some(b"Exif\0\0garbage that is not TIFF".to_vec());
    source.xmp = None;
    let output = jpeg::encode(&source, jpeg::JpegOptions::default())
        .unwrap_or_else(|error| panic!("{error:?}"));
    assert!(output.adjustments.metadata_dropped);
    assert!(!contains(&output.bytes, b"garbage that is not TIFF"));
}

#[test]
fn sources_without_metadata_report_nothing_dropped() {
    let mut source = camera_source();
    source.exif = None;
    source.xmp = None;
    let output = webp::encode(&source, webp::WebpOptions::default())
        .unwrap_or_else(|error| panic!("{error:?}"));
    assert!(!output.adjustments.metadata_dropped);
}

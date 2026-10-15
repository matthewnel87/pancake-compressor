//! M1 format matrix: every supported input format through every encoder.
//!
//! This is the evidence for the M1 gate in `docs/17-codex-execution-plan.md`
//! ("each supported input/output mapping has a passing or documented
//! explicitly-unsupported test"). The table it checks is documented in
//! `docs/07-image-format-contract.md` (section "M1 implemented matrix").
//!
//! Inputs are made by this crate's own encoders from one synthetic RGBA image
//! with a transparent area, so every path starts from the same picture.

// Test code: a panic is the failure report.
#![allow(clippy::panic)]

use std::io::Cursor;

use image::{DynamicImage, ImageBuffer, Rgba};
use pancake_core::decode::{DecodedImage, decode};
use pancake_core::encode::{EncodeError, avif, jpeg, png, webp};
use pancake_core::format::ImageFormat;
use pancake_core::limits::ImageLimits;

const W: u32 = 48;
const H: u32 = 32;

/// Gradient with a fully transparent left third and a half-transparent band.
fn picture() -> DynamicImage {
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(W, H, |x, y| {
        let alpha = if x < W / 3 {
            0
        } else if y < 4 {
            128
        } else {
            255
        };
        Rgba([(x * 255 / W) as u8, (y * 255 / H) as u8, 140, alpha])
    }))
}

fn open(bytes: &[u8]) -> DecodedImage {
    decode(bytes, &ImageLimits::default())
        .unwrap_or_else(|error| panic!("decode failed: {error:?}"))
}

fn transparent(image: &DynamicImage) -> bool {
    image.to_rgba8().pixels().any(|p| p[3] < 250)
}

/// One input file per supported format/mode.
fn inputs() -> Vec<(&'static str, Vec<u8>)> {
    let mut png_bytes = Cursor::new(Vec::new());
    picture()
        .write_to(&mut png_bytes, image::ImageFormat::Png)
        .unwrap_or_else(|error| panic!("{error}"));
    let png_bytes = png_bytes.into_inner();
    let source = open(&png_bytes);
    let fast_avif = avif::AvifOptions {
        speed: 10,
        ..avif::AvifOptions::default()
    };
    let lossless_webp = webp::WebpOptions {
        lossless: true,
        ..webp::WebpOptions::default()
    };
    let ok = |result: Result<pancake_core::encode::Encoded, EncodeError>| {
        result
            .unwrap_or_else(|error| panic!("fixture encode failed: {error:?}"))
            .bytes
    };
    vec![
        ("png", png_bytes.clone()),
        (
            "jpeg",
            ok(jpeg::encode(&source, jpeg::JpegOptions::default())),
        ),
        (
            "webp-lossy",
            ok(webp::encode(&source, webp::WebpOptions::default())),
        ),
        ("webp-lossless", ok(webp::encode(&source, lossless_webp))),
        ("avif", ok(avif::encode(&source, fast_avif))),
    ]
}

#[test]
fn every_input_format_converts_to_every_output() {
    for (input_name, bytes) in inputs() {
        let source = open(&bytes);
        let input_has_alpha = transparent(&source.pixels);
        // JPEG inputs are opaque by nature; all others carry the cut-out.
        assert_eq!(input_has_alpha, input_name != "jpeg", "{input_name}");

        // PNG lossless works on original PNG bytes only (it is a
        // recompressor, not a converter). Documented in docs/07.
        let lossless = png::optimise_lossless(&bytes, &source, png::LosslessOptions::default());
        if source.info.format == ImageFormat::Png {
            let out =
                lossless.unwrap_or_else(|error| panic!("{input_name} -> png lossless: {error:?}"));
            assert_eq!(open(&out).pixels.to_rgba16(), source.pixels.to_rgba16());
        } else {
            assert!(
                matches!(lossless, Err(EncodeError::InvalidOptions(_))),
                "{input_name} -> png lossless must refuse non-PNG sources"
            );
        }

        let palette = png::quantise_lossy(&source, png::LossyOptions::default());
        match palette {
            Ok(out) => {
                let back = open(&out);
                assert_eq!((back.info.width, back.info.height), (W, H));
                assert_eq!(
                    transparent(&back.pixels),
                    input_has_alpha,
                    "{input_name} -> png palette"
                );
            }
            // A legitimate, explicit outcome: the quality floor was not met.
            Err(EncodeError::QualityNotReached(_)) => {}
            Err(error) => panic!("{input_name} -> png palette: {error:?}"),
        }

        let to_jpeg = jpeg::encode(&source, jpeg::JpegOptions::default())
            .unwrap_or_else(|error| panic!("{input_name} -> jpeg: {error:?}"));
        assert_eq!(
            to_jpeg.adjustments.transparency_flattened, input_has_alpha,
            "{input_name} -> jpeg"
        );
        assert!(!transparent(&open(&to_jpeg.bytes).pixels));

        for lossless in [false, true] {
            let options = webp::WebpOptions {
                lossless,
                ..webp::WebpOptions::default()
            };
            let out = webp::encode(&source, options).unwrap_or_else(|error| {
                panic!("{input_name} -> webp lossless={lossless}: {error:?}")
            });
            let back = open(&out.bytes);
            assert_eq!(
                transparent(&back.pixels),
                input_has_alpha,
                "{input_name} -> webp"
            );
            if lossless {
                assert_eq!(
                    back.pixels.to_rgba8(),
                    source.pixels.to_rgba8(),
                    "{input_name} -> webp lossless"
                );
            }
        }

        let out = avif::encode(
            &source,
            avif::AvifOptions {
                speed: 10,
                ..avif::AvifOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("{input_name} -> avif: {error:?}"));
        let back = open(&out.bytes);
        assert_eq!((back.info.width, back.info.height), (W, H));
        assert_eq!(
            transparent(&back.pixels),
            input_has_alpha,
            "{input_name} -> avif"
        );
    }
}

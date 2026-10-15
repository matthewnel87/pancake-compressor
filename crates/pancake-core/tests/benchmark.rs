//! Opt-in size/time benchmark for the M1 encoders.
//!
//! Ignored by default (it takes seconds and its numbers depend on the
//! machine). Run in release mode:
//!
//! ```text
//! cargo test -p pancake-core --release --test benchmark -- --ignored --nocapture
//! ```
//!
//! The input is a **synthetic** 4000 x 3000 "photo" (smooth gradients plus
//! deterministic grain and a few hard edges). Real photographs compress
//! differently, so these numbers compare encoders and machines; they are not
//! a claim about savings on users' images (docs/11, docs/22 copy rules).

#![allow(clippy::panic)]

use std::io::Cursor;
use std::time::Instant;

use image::{DynamicImage, ImageBuffer, Rgb};
use pancake_core::decode::decode;
use pancake_core::encode::{avif, jpeg, png, webp};
use pancake_core::limits::ImageLimits;

fn synthetic_photo(width: u32, height: u32) -> DynamicImage {
    let mut state: u32 = 0x5EED_1234;
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(width, height, |x, y| {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let grain = (state % 12) as u8;
        let edge = if (x / 500 + y / 500) % 2 == 0 { 30 } else { 0 };
        Rgb([
            ((x * 180 / width) as u8 + edge).saturating_add(grain),
            ((y * 180 / height) as u8).saturating_add(grain),
            (90 + edge).saturating_add(grain / 2),
        ])
    }))
}

#[test]
#[ignore = "benchmark: run explicitly in release mode"]
fn encoder_size_and_time() {
    let (width, height) = (4000, 3000);
    let mut png_bytes = Cursor::new(Vec::new());
    synthetic_photo(width, height)
        .write_to(&mut png_bytes, image::ImageFormat::Png)
        .unwrap_or_else(|error| panic!("{error}"));
    let png_bytes = png_bytes.into_inner();

    let started = Instant::now();
    let source =
        decode(&png_bytes, &ImageLimits::default()).unwrap_or_else(|error| panic!("{error:?}"));
    println!(
        "input: synthetic {width}x{height} PNG, {} bytes",
        png_bytes.len()
    );
    println!("decode png: {:.2}s", started.elapsed().as_secs_f64());

    let report = |name: &str, run: &mut dyn FnMut() -> Vec<u8>| {
        let started = Instant::now();
        let bytes = run();
        println!(
            "{name:<24} {:>10} bytes  {:>6.1}% of PNG  {:>6.2}s",
            bytes.len(),
            bytes.len() as f64 * 100.0 / png_bytes.len() as f64,
            started.elapsed().as_secs_f64()
        );
    };
    report("png lossless (effort 2)", &mut || {
        png::optimise_lossless(&png_bytes, &source, png::LosslessOptions::default())
            .unwrap_or_else(|error| panic!("{error:?}"))
    });
    report("png palette (default)", &mut || {
        png::quantise_lossy(&source, png::LossyOptions::default())
            .unwrap_or_else(|error| panic!("{error:?}"))
    });
    report("jpeg q80 4:2:0", &mut || {
        jpeg::encode(&source, jpeg::JpegOptions::default())
            .unwrap_or_else(|error| panic!("{error:?}"))
            .bytes
    });
    report("webp lossy q80 m4", &mut || {
        webp::encode(&source, webp::WebpOptions::default())
            .unwrap_or_else(|error| panic!("{error:?}"))
            .bytes
    });
    report("webp lossless", &mut || {
        webp::encode(
            &source,
            webp::WebpOptions {
                lossless: true,
                ..webp::WebpOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("{error:?}"))
        .bytes
    });
    report("avif q70 speed 6, 1 thr", &mut || {
        avif::encode(&source, avif::AvifOptions::default())
            .unwrap_or_else(|error| panic!("{error:?}"))
            .bytes
    });
}

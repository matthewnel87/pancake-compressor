//! Tests for header inspection.
//!
//! Every fixture is built byte by byte here, so its content is fully known and
//! redistributable (docs/11 fixture rules). Only header bytes are synthesised:
//! checksums and pixel data are not valid, which is fine because this module
//! never decodes pixels or verifies checksums.

use super::*;

// ---------------------------------------------------------------- builders

fn png_chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut out = (data.len() as u32).to_be_bytes().to_vec();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&[0, 0, 0, 0]); // CRC, not checked by inspect()
    out
}

fn png(width: u32, height: u32, colour_type: u8, before_idat: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
    let mut ihdr = width.to_be_bytes().to_vec();
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, colour_type, 0, 0, 0]);
    let mut out = PNG_SIGNATURE.to_vec();
    out.extend(png_chunk(b"IHDR", &ihdr));
    for (kind, data) in before_idat {
        out.extend(png_chunk(kind, data));
    }
    out.extend(png_chunk(b"IDAT", &[0; 4]));
    out.extend(png_chunk(b"IEND", &[]));
    out
}

/// JPEG with a JFIF APP0 segment followed by a frame header using `sof`.
fn jpeg(sof: u8, width: u16, height: u16) -> Vec<u8> {
    let mut out = vec![0xFF, 0xD8];
    out.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x10]);
    out.extend_from_slice(b"JFIF\0\x01\x02\x00\x00\x01\x00\x01\x00\x00");
    out.extend_from_slice(&[0xFF, sof, 0x00, 0x11, 0x08]);
    out.extend_from_slice(&height.to_be_bytes());
    out.extend_from_slice(&width.to_be_bytes());
    out.extend_from_slice(&[0x03, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1]);
    out.extend_from_slice(&[0xFF, 0xD9]);
    out
}

fn riff_webp(chunk: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut body = b"WEBP".to_vec();
    body.extend_from_slice(chunk);
    body.extend_from_slice(&(data.len() as u32).to_le_bytes());
    body.extend_from_slice(data);
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend(body);
    out
}

fn webp_lossy(width: u16, height: u16) -> Vec<u8> {
    let mut data = vec![0x30, 0x01, 0x00, 0x9D, 0x01, 0x2A];
    data.extend_from_slice(&width.to_le_bytes());
    data.extend_from_slice(&height.to_le_bytes());
    data.extend_from_slice(&[0; 8]);
    riff_webp(b"VP8 ", &data)
}

fn webp_lossless(width: u32, height: u32, alpha: bool) -> Vec<u8> {
    let bits = (width - 1) | ((height - 1) << 14) | (u32::from(alpha) << 28);
    let mut data = vec![0x2F];
    data.extend_from_slice(&bits.to_le_bytes());
    data.extend_from_slice(&[0; 4]);
    riff_webp(b"VP8L", &data)
}

fn webp_extended(flags: u8, width: u32, height: u32) -> Vec<u8> {
    let mut data = vec![flags, 0, 0, 0];
    data.extend_from_slice(&(width - 1).to_le_bytes()[..3]);
    data.extend_from_slice(&(height - 1).to_le_bytes()[..3]);
    riff_webp(b"VP8X", &data)
}

fn iso_box(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
    out.extend_from_slice(kind);
    out.extend_from_slice(payload);
    out
}

fn ispe(width: u32, height: u32) -> Vec<u8> {
    let mut payload = vec![0, 0, 0, 0]; // FullBox version + flags
    payload.extend_from_slice(&width.to_be_bytes());
    payload.extend_from_slice(&height.to_be_bytes());
    iso_box(b"ispe", &payload)
}

fn avif_like(major: &[u8; 4], compatible: &[&[u8; 4]], sizes: &[(u32, u32)]) -> Vec<u8> {
    let mut ftyp = major.to_vec();
    ftyp.extend_from_slice(&[0, 0, 0, 0]);
    for brand in compatible {
        ftyp.extend_from_slice(*brand);
    }
    let properties: Vec<u8> = sizes.iter().flat_map(|(w, h)| ispe(*w, *h)).collect();
    let ipco = iso_box(b"ipco", &properties);
    let iprp = iso_box(b"iprp", &ipco);
    let mut meta_payload = vec![0, 0, 0, 0];
    meta_payload.extend(iso_box(b"hdlr", &[0; 25]));
    meta_payload.extend(iprp);
    let mut out = iso_box(b"ftyp", &ftyp);
    out.extend(iso_box(b"meta", &meta_payload));
    out.extend(iso_box(b"mdat", &[0; 16]));
    out
}

fn avif(width: u32, height: u32) -> Vec<u8> {
    avif_like(b"avif", &[b"mif1", b"avif"], &[(width, height)])
}

fn info(format: ImageFormat, width: u32, height: u32, alpha: AlphaHint) -> ImageInfo {
    ImageInfo {
        format,
        width,
        height,
        alpha,
    }
}

// ------------------------------------------------------------- happy paths

#[test]
fn png_reports_dimensions_and_alpha_from_colour_type() {
    let rgba = png(640, 480, 6, &[]);
    assert_eq!(
        inspect(&rgba),
        Ok(info(ImageFormat::Png, 640, 480, AlphaHint::Present))
    );
    let rgb = png(1, 2, 2, &[]);
    assert_eq!(
        inspect(&rgb),
        Ok(info(ImageFormat::Png, 1, 2, AlphaHint::Absent))
    );
}

#[test]
fn png_transparency_chunk_means_alpha() {
    let palette_with_trns = png(16, 16, 3, &[(b"PLTE", &[0; 6]), (b"tRNS", &[0])]);
    assert_eq!(
        inspect(&palette_with_trns).map(|i| i.alpha),
        Ok(AlphaHint::Present)
    );
}

#[test]
fn jpeg_reports_dimensions_for_baseline_and_progressive() {
    for sof in [0xC0, 0xC1, 0xC2] {
        assert_eq!(
            inspect(&jpeg(sof, 4000, 3000)),
            Ok(info(ImageFormat::Jpeg, 4000, 3000, AlphaHint::Absent)),
            "SOF marker {sof:#04X}"
        );
    }
}

#[test]
fn jpeg_tolerates_fill_bytes_before_a_marker() {
    let mut bytes = jpeg(0xC0, 10, 20);
    // Insert extra 0xFF fill bytes before the APP0 marker code.
    bytes.splice(2..3, [0xFF, 0xFF, 0xFF]);
    assert_eq!(inspect(&bytes).map(|i| (i.width, i.height)), Ok((10, 20)));
}

#[test]
fn webp_variants_report_dimensions_and_alpha() {
    assert_eq!(
        inspect(&webp_lossy(300, 200)),
        Ok(info(ImageFormat::WebP, 300, 200, AlphaHint::Absent))
    );
    assert_eq!(
        inspect(&webp_lossless(16384, 1, true)),
        Ok(info(ImageFormat::WebP, 16384, 1, AlphaHint::Present))
    );
    assert_eq!(
        inspect(&webp_extended(0x10, 5000, 7000)),
        Ok(info(ImageFormat::WebP, 5000, 7000, AlphaHint::Present))
    );
}

#[test]
fn avif_reports_largest_declared_size() {
    // Primary image plus a smaller alpha/thumbnail item: the larger one wins
    // because it bounds the memory the decoder may need.
    let bytes = avif_like(b"avif", &[b"mif1"], &[(64, 64), (1920, 1080), (320, 180)]);
    assert_eq!(
        inspect(&bytes),
        Ok(info(ImageFormat::Avif, 1920, 1080, AlphaHint::Unknown))
    );
    assert_eq!(inspect(&avif(1, 1)).map(|i| i.width), Ok(1));
}

// ------------------------------------------------------------- animation

#[test]
fn animated_inputs_are_refused_not_flattened() {
    let apng = png(32, 32, 6, &[(b"acTL", &[0, 0, 0, 2, 0, 0, 0, 0])]);
    assert_eq!(
        inspect(&apng),
        Err(InspectError::Animated(ImageFormat::Png))
    );

    let animated_webp = webp_extended(0x02 | 0x10, 32, 32);
    assert_eq!(
        inspect(&animated_webp),
        Err(InspectError::Animated(ImageFormat::WebP))
    );

    // Sequence brand alongside the still brand: still refused.
    let sequence = avif_like(b"avis", &[b"avif", b"msf1"], &[(32, 32)]);
    assert_eq!(
        inspect(&sequence),
        Err(InspectError::Animated(ImageFormat::Avif))
    );
}

// ------------------------------------------------------ unsupported formats

#[test]
fn unsupported_formats_are_named() {
    let cases: [(&[u8], UnsupportedKind); 5] = [
        (b"GIF89a\x01\x00\x01\x00", UnsupportedKind::Gif),
        (b"BM\0\0\0\0\0\0\0\0\0\0\0\0", UnsupportedKind::Bmp),
        (b"II*\0\x08\0\0\0", UnsupportedKind::Tiff),
        (&[0xFF, 0x0A, 0, 0], UnsupportedKind::JpegXl),
        (&jpeg(0xC3, 8, 8), UnsupportedKind::JpegCodingProcess),
    ];
    for (bytes, kind) in cases {
        assert_eq!(inspect(bytes), Err(InspectError::Unsupported(kind)));
    }
    let heic = avif_like(b"heic", &[b"mif1", b"heic"], &[(8, 8)]);
    assert_eq!(
        inspect(&heic),
        Err(InspectError::Unsupported(UnsupportedKind::Heif))
    );
}

#[test]
fn extension_is_ignored_in_favour_of_content() {
    // A WebP saved as ".png" is still a WebP. Callers may warn about the
    // mismatch but must process (or refuse) it by its real format.
    assert_eq!(ImageFormat::from_extension("PNG"), Some(ImageFormat::Png));
    assert_eq!(
        inspect(&webp_lossy(1, 1)).map(|i| i.format),
        Ok(ImageFormat::WebP)
    );
    assert_eq!(ImageFormat::from_extension("JPEG"), Some(ImageFormat::Jpeg));
    assert_eq!(ImageFormat::from_extension("gif"), None);
}

// ----------------------------------------------------- malformed and hostile

#[test]
fn empty_and_unknown_inputs() {
    assert_eq!(inspect(&[]), Err(InspectError::Empty));
    assert_eq!(
        inspect(b"hello, not an image"),
        Err(InspectError::UnknownFormat)
    );
}

#[test]
fn zero_dimensions_are_refused() {
    assert_eq!(
        inspect(&png(0, 10, 6, &[])),
        Err(InspectError::ZeroDimensions(ImageFormat::Png))
    );
    // JPEG height 0 means "defined later by DNL", which is not supported.
    assert_eq!(
        inspect(&jpeg(0xC0, 10, 0)),
        Err(InspectError::ZeroDimensions(ImageFormat::Jpeg))
    );
    assert_eq!(
        inspect(&avif(0, 5)),
        Err(InspectError::ZeroDimensions(ImageFormat::Avif))
    );
}

#[test]
fn structural_errors_are_reported_not_panicked() {
    let mut not_ihdr = png(1, 1, 6, &[]);
    not_ihdr[12..16].copy_from_slice(b"IDAT");
    assert!(matches!(
        inspect(&not_ihdr),
        Err(InspectError::Malformed(ImageFormat::Png, _))
    ));

    // A chunk claiming ~4 GB jumps past the end of the file.
    let mut huge_chunk = png(1, 1, 6, &[(b"tEXt", &[0; 4])]);
    huge_chunk[33..37].copy_from_slice(&0xFFFF_FFF0u32.to_be_bytes());
    assert_eq!(
        inspect(&huge_chunk),
        Err(InspectError::Truncated(ImageFormat::Png))
    );

    let scan_first = [0xFF, 0xD8, 0xFF, 0xDA, 0x00, 0x08, 0, 0, 0, 0, 0, 0];
    assert!(matches!(
        inspect(&scan_first),
        Err(InspectError::Malformed(ImageFormat::Jpeg, _))
    ));

    let mut bad_segment = jpeg(0xC0, 1, 1);
    bad_segment[4..6].copy_from_slice(&[0, 1]); // length below 2
    assert!(matches!(
        inspect(&bad_segment),
        Err(InspectError::Malformed(ImageFormat::Jpeg, _))
    ));

    let no_meta = iso_box(b"ftyp", b"avif\0\0\0\0mif1");
    assert!(matches!(
        inspect(&no_meta),
        Err(InspectError::Malformed(ImageFormat::Avif, _))
    ));
}

#[test]
fn many_tiny_structures_stop_at_the_cap() {
    // Thousands of empty ancillary chunks before IDAT: bounded, not a hang.
    let filler: Vec<(&[u8; 4], &[u8])> = (0..MAX_STRUCTURES + 10)
        .map(|_| (b"zzZz", &[][..]))
        .collect();
    let bytes = png(1, 1, 6, &filler);
    assert!(matches!(
        inspect(&bytes),
        Err(InspectError::Malformed(ImageFormat::Png, _))
    ));
}

#[test]
fn every_truncation_of_every_fixture_is_handled() {
    let fixtures = [
        png(640, 480, 6, &[(b"tRNS", &[0])]),
        jpeg(0xC2, 4000, 3000),
        webp_lossy(300, 200),
        webp_lossless(16, 16, true),
        webp_extended(0x10, 64, 64),
        avif(1920, 1080),
    ];
    for fixture in &fixtures {
        for end in 0..fixture.len() {
            // Must return (any result) without panicking.
            let _ = inspect(&fixture[..end]);
        }
    }
}

#[test]
fn random_bytes_after_real_signatures_never_panic() {
    // Small deterministic generator, so failures are reproducible.
    let mut state: u64 = 0x5EED_CAFE_F00D_0001;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let prefixes: [&[u8]; 5] = [
        PNG_SIGNATURE,
        &[0xFF, 0xD8, 0xFF],
        b"RIFF\0\0\0\0WEBP",
        b"\0\0\0\x14ftypavif",
        b"",
    ];
    for round in 0..20_000 {
        let prefix = prefixes[round % prefixes.len()];
        let len = (next() % 96) as usize;
        let mut bytes = prefix.to_vec();
        bytes.extend((0..len).map(|_| next() as u8));
        let _ = inspect(&bytes);
    }
}

#[test]
fn error_codes_are_unique_and_messages_mention_no_bytes() {
    let errors = [
        InspectError::Empty,
        InspectError::UnknownFormat,
        InspectError::Unsupported(UnsupportedKind::Gif),
        InspectError::Animated(ImageFormat::WebP),
        InspectError::Truncated(ImageFormat::Png),
        InspectError::Malformed(ImageFormat::Jpeg, "internal detail"),
        InspectError::ZeroDimensions(ImageFormat::Avif),
        InspectError::TooLarge {
            format: ImageFormat::Png,
            width: 1,
            height: 1,
        },
    ];
    let mut codes: Vec<&str> = errors.iter().map(InspectError::code).collect();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), errors.len());
    // The developer detail in Malformed is not shown to users.
    let shown = InspectError::Malformed(ImageFormat::Jpeg, "internal detail").to_string();
    assert!(!shown.contains("internal detail"));
}

#[test]
fn preflight_refuses_decompression_bombs_before_decoding() {
    let limits = ImageLimits::default();
    // A ~60-byte PNG header claiming 100,000 x 100,000 pixels (40 GB as RGBA).
    let bomb = png(100_000, 100_000, 6, &[]);
    assert!(bomb.len() < 100);
    assert_eq!(
        preflight(&bomb, &limits),
        Err(InspectError::TooLarge {
            format: ImageFormat::Png,
            width: 100_000,
            height: 100_000
        })
    );
    // AVIF declares sizes in 32-bit fields, so it can claim even more.
    let avif_bomb = avif(u32::MAX, u32::MAX);
    assert_eq!(
        preflight(&avif_bomb, &limits).map_err(|e| e.code()),
        Err("too-many-pixels")
    );
    // Ordinary images pass unchanged.
    assert_eq!(
        preflight(&jpeg(0xC0, 4000, 3000), &limits),
        inspect(&jpeg(0xC0, 4000, 3000))
    );
}

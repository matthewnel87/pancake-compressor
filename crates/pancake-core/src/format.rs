//! Header-only identification of image files.
//!
//! Before any decoder sees a file, this module answers four questions from
//! the first bytes alone: what format it really is (magic bytes, never the
//! file extension), how many pixels it claims to have, whether it may carry
//! transparency, and whether it is animated.
//!
//! **Why header-only:** decoders allocate memory based on the dimensions a
//! file declares. Reading those dimensions here lets the caller apply
//! [`crate::limits`] *before* a decoder can be tricked into a huge
//! allocation (a "decompression bomb", see `docs/10-threat-model.md`).
//!
//! **What it does not own:** pixel data, colour profiles and metadata
//! correctness. A header can be valid while the rest of the file is corrupt;
//! the decoder must still fail safely on that. Checksums (PNG CRCs, for
//! example) are not verified here for the same reason.
//!
//! **Animation policy** (`docs/07-image-format-contract.md`): v0.1 handles
//! still images only. Animated PNG, animated WebP and AVIF image sequences
//! are reported as [`InspectError::Animated`] so the app can skip them with
//! a clear reason instead of silently keeping only the first frame.
//!
//! Every read goes through bounds-checked helpers, and the parsers walk a
//! bounded number of chunks or boxes, so hostile input yields an error rather
//! than a panic or an unbounded loop.

use std::fmt;

use crate::limits::ImageLimits;

/// Still-image formats the engine intends to read and write in v0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Avif,
}

impl ImageFormat {
    /// Short lowercase name, stable for logs and IPC (`"png"`, `"jpeg"` ...).
    pub fn as_str(self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::WebP => "webp",
            ImageFormat::Avif => "avif",
        }
    }

    /// Format usually associated with a file extension (case-insensitive,
    /// without the dot). Only a hint: [`inspect`] decides the real format.
    pub fn from_extension(extension: &str) -> Option<ImageFormat> {
        match extension.to_ascii_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" | "jpe" | "jfif" => Some(ImageFormat::Jpeg),
            "webp" => Some(ImageFormat::WebP),
            "avif" => Some(ImageFormat::Avif),
            _ => None,
        }
    }
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpeg => "JPEG",
            ImageFormat::WebP => "WebP",
            ImageFormat::Avif => "AVIF",
        };
        f.write_str(name)
    }
}

/// What the header says about transparency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaHint {
    /// The header declares an alpha channel or transparency chunk.
    Present,
    /// The format variant cannot carry alpha (for example JPEG, lossy WebP
    /// without an extended header).
    Absent,
    /// The header cannot say; only decoding will tell (AVIF alpha planes).
    Unknown,
}

/// Facts read from a still image's header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageInfo {
    pub format: ImageFormat,
    /// Declared width in pixels. Untrusted until the decoder agrees.
    pub width: u32,
    /// Declared height in pixels. Untrusted until the decoder agrees.
    pub height: u32,
    pub alpha: AlphaHint,
}

/// Recognised formats that v0.1 deliberately does not process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedKind {
    Gif,
    Bmp,
    Tiff,
    /// HEIC/HEIF and other ISO-BMFF images that are not AVIF.
    Heif,
    JpegXl,
    /// JPEG using lossless, hierarchical or arithmetic coding, which common
    /// decoders do not support reliably.
    JpegCodingProcess,
}

/// Why a file was not accepted. Each variant maps to a stable
/// [`InspectError::code`] for the UI and IPC events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectError {
    /// Zero-byte file.
    Empty,
    /// No known image signature.
    UnknownFormat,
    /// A recognised format that v0.1 does not process.
    Unsupported(UnsupportedKind),
    /// More than one frame; refused rather than silently flattened.
    Animated(ImageFormat),
    /// The file ends before the header is complete.
    Truncated(ImageFormat),
    /// The header is internally inconsistent. The `&'static str` names the
    /// rule broken, for developers; it never contains file content.
    Malformed(ImageFormat, &'static str),
    /// Width or height of zero.
    ZeroDimensions(ImageFormat),
    /// Declared size exceeds [`ImageLimits`]; refused before decoding.
    TooLarge {
        format: ImageFormat,
        width: u32,
        height: u32,
    },
}

impl InspectError {
    /// Stable machine-readable code, used as `errorCode` in progress events.
    pub fn code(&self) -> &'static str {
        match self {
            InspectError::Empty => "empty-file",
            InspectError::UnknownFormat => "unknown-format",
            InspectError::Unsupported(_) => "unsupported-format",
            InspectError::Animated(_) => "animated-image",
            InspectError::Truncated(_) => "truncated-header",
            InspectError::Malformed(_, _) => "malformed-header",
            InspectError::ZeroDimensions(_) => "zero-dimensions",
            InspectError::TooLarge { .. } => "too-many-pixels",
        }
    }
}

impl fmt::Display for InspectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InspectError::Empty => f.write_str("The file is empty."),
            InspectError::UnknownFormat => {
                f.write_str("This file is not a PNG, JPEG, WebP or AVIF image.")
            }
            InspectError::Unsupported(kind) => {
                let what = match kind {
                    UnsupportedKind::Gif => "GIF images are not supported yet.",
                    UnsupportedKind::Bmp => "BMP images are not supported.",
                    UnsupportedKind::Tiff => "TIFF images are not supported.",
                    UnsupportedKind::Heif => "HEIC/HEIF images are not supported yet.",
                    UnsupportedKind::JpegXl => "JPEG XL images are not supported yet.",
                    UnsupportedKind::JpegCodingProcess => {
                        "This JPEG uses an uncommon encoding that is not supported."
                    }
                };
                f.write_str(what)
            }
            InspectError::Animated(format) => write!(
                f,
                "This is an animated {format}. Only still images are supported, so it was skipped rather than cut down to one frame."
            ),
            InspectError::Truncated(format) => {
                write!(f, "This {format} file is incomplete or cut short.")
            }
            InspectError::Malformed(format, _) => {
                write!(f, "This {format} file appears to be damaged.")
            }
            InspectError::ZeroDimensions(format) => {
                write!(f, "This {format} file says it has no pixels.")
            }
            InspectError::TooLarge {
                format,
                width,
                height,
            } => write!(
                f,
                "This {format} is {width} × {height} pixels, which is larger than Pancake Compressor can safely process."
            ),
        }
    }
}

impl std::error::Error for InspectError {}

/// Identifies an image from its bytes and reads its header.
///
/// `bytes` should be the whole file, or at least a prefix long enough to
/// contain the header (PNG and WebP headers are near the start; JPEG and
/// AVIF headers can follow large metadata blocks). A prefix that is too short
/// is reported as [`InspectError::Truncated`].
///
/// ```
/// use pancake_core::format::{inspect, InspectError};
/// assert_eq!(inspect(b"GIF89a....").unwrap_err().code(), "unsupported-format");
/// assert_eq!(inspect(b""), Err(InspectError::Empty));
/// ```
pub fn inspect(bytes: &[u8]) -> Result<ImageInfo, InspectError> {
    if bytes.is_empty() {
        return Err(InspectError::Empty);
    }
    if bytes.starts_with(PNG_SIGNATURE) {
        return check_dimensions(png::inspect(bytes)?);
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return check_dimensions(jpeg::inspect(bytes)?);
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        return check_dimensions(webp::inspect(bytes)?);
    }
    if bytes.get(4..8) == Some(b"ftyp") {
        return check_dimensions(isobmff::inspect(bytes)?);
    }
    Err(identify_unsupported(bytes))
}

/// [`inspect`] followed by the pixel caps in `limits`.
///
/// This is the check every file must pass before it is handed to a decoder.
///
/// ```
/// use pancake_core::{format::preflight, limits::ImageLimits};
/// // A 1x1 PNG header (IHDR only; pixel data is not needed for preflight).
/// let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
/// png.extend([0, 0, 0, 13]);
/// png.extend(b"IHDR");
/// png.extend([0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0, 0, 0, 0]);
/// png.extend([0, 0, 0, 0]);
/// png.extend(b"IDAT");
/// assert!(preflight(&png, &ImageLimits::default()).is_ok());
/// ```
pub fn preflight(bytes: &[u8], limits: &ImageLimits) -> Result<ImageInfo, InspectError> {
    let info = inspect(bytes)?;
    if limits.allows(info.width, info.height) {
        Ok(info)
    } else {
        Err(InspectError::TooLarge {
            format: info.format,
            width: info.width,
            height: info.height,
        })
    }
}

fn check_dimensions(info: ImageInfo) -> Result<ImageInfo, InspectError> {
    if info.width == 0 || info.height == 0 {
        Err(InspectError::ZeroDimensions(info.format))
    } else {
        Ok(info)
    }
}

fn identify_unsupported(bytes: &[u8]) -> InspectError {
    let kind = if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some(UnsupportedKind::Gif)
    } else if bytes.starts_with(b"BM") && bytes.len() >= 14 {
        Some(UnsupportedKind::Bmp)
    } else if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        Some(UnsupportedKind::Tiff)
    } else if bytes.starts_with(&[0xFF, 0x0A])
        || bytes.starts_with(&[0, 0, 0, 0x0C, b'J', b'X', b'L', b' '])
    {
        Some(UnsupportedKind::JpegXl)
    } else {
        None
    };
    kind.map_or(InspectError::UnknownFormat, InspectError::Unsupported)
}

const PNG_SIGNATURE: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Upper bound on chunks/segments/boxes walked while looking for a header.
/// Real files need a handful; the cap stops crafted files with thousands of
/// tiny chunks from turning inspection into a slow loop.
const MAX_STRUCTURES: usize = 4096;

// Bounds-checked big/little-endian readers. Each returns `None` instead of
// panicking when the slice is too short.
fn be_u16(bytes: &[u8], at: usize) -> Option<u16> {
    let b = bytes.get(at..at.checked_add(2)?)?;
    Some(u16::from_be_bytes([b[0], b[1]]))
}

fn be_u32(bytes: &[u8], at: usize) -> Option<u32> {
    let b = bytes.get(at..at.checked_add(4)?)?;
    Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

fn be_u64(bytes: &[u8], at: usize) -> Option<u64> {
    let b = bytes.get(at..at.checked_add(8)?)?;
    Some(u64::from_be_bytes([
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
    ]))
}

fn le_u24(bytes: &[u8], at: usize) -> Option<u32> {
    let b = bytes.get(at..at.checked_add(3)?)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], 0]))
}

fn le_u32(bytes: &[u8], at: usize) -> Option<u32> {
    let b = bytes.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// PNG: <http://www.libpng.org/pub/png/spec/1.2/PNG-Structures.html> and the
/// APNG extension <https://wiki.mozilla.org/APNG_Specification>.
mod png {
    use super::*;

    const F: ImageFormat = ImageFormat::Png;

    pub(super) fn inspect(bytes: &[u8]) -> Result<ImageInfo, InspectError> {
        // IHDR must be the first chunk, immediately after the signature.
        let ihdr_len = be_u32(bytes, 8).ok_or(InspectError::Truncated(F))?;
        if bytes.get(12..16).ok_or(InspectError::Truncated(F))? != b"IHDR" {
            return Err(InspectError::Malformed(F, "first chunk is not IHDR"));
        }
        if ihdr_len != 13 {
            return Err(InspectError::Malformed(F, "IHDR length is not 13"));
        }
        let width = be_u32(bytes, 16).ok_or(InspectError::Truncated(F))?;
        let height = be_u32(bytes, 20).ok_or(InspectError::Truncated(F))?;
        let colour_type = *bytes.get(25).ok_or(InspectError::Truncated(F))?;
        let mut alpha = match colour_type {
            4 | 6 => AlphaHint::Present,    // grey+alpha, RGBA
            0 | 2 | 3 => AlphaHint::Absent, // may change if tRNS follows
            _ => return Err(InspectError::Malformed(F, "unknown colour type")),
        };

        // Walk chunks up to the first IDAT. APNG requires `acTL` before the
        // image data, and transparency (`tRNS`) must also precede it.
        let mut offset: usize = 8 + 12 + 13; // signature + IHDR header/data/CRC
        for _ in 0..MAX_STRUCTURES {
            let length = be_u32(bytes, offset).ok_or(InspectError::Truncated(F))?;
            let kind = bytes
                .get(offset + 4..offset + 8)
                .ok_or(InspectError::Truncated(F))?;
            match kind {
                b"acTL" => return Err(InspectError::Animated(F)),
                b"tRNS" => alpha = AlphaHint::Present,
                b"IDAT" | b"IEND" => {
                    return Ok(ImageInfo {
                        format: F,
                        width,
                        height,
                        alpha,
                    });
                }
                _ => {}
            }
            // length (4) + type (4) + data + CRC (4), all checked.
            let step = usize::try_from(length)
                .ok()
                .and_then(|len| len.checked_add(12))
                .ok_or(InspectError::Malformed(F, "chunk length overflows"))?;
            offset = offset
                .checked_add(step)
                .ok_or(InspectError::Malformed(F, "chunk length overflows"))?;
        }
        Err(InspectError::Malformed(
            F,
            "too many chunks before image data",
        ))
    }
}

/// JPEG: ITU-T T.81 section B. Walks marker segments until the first
/// start-of-frame (SOF) marker, which carries the dimensions.
mod jpeg {
    use super::*;

    const F: ImageFormat = ImageFormat::Jpeg;

    pub(super) fn inspect(bytes: &[u8]) -> Result<ImageInfo, InspectError> {
        let mut offset: usize = 2; // after SOI (FF D8)
        for _ in 0..MAX_STRUCTURES {
            // Each marker is 0xFF followed by a code; extra 0xFF fill bytes
            // are allowed before the code.
            if *bytes.get(offset).ok_or(InspectError::Truncated(F))? != 0xFF {
                return Err(InspectError::Malformed(F, "expected a marker"));
            }
            let mut marker_at = offset + 1;
            let mut marker = *bytes.get(marker_at).ok_or(InspectError::Truncated(F))?;
            let mut fill = 0usize;
            while marker == 0xFF {
                fill += 1;
                if fill > 64 {
                    return Err(InspectError::Malformed(F, "too many fill bytes"));
                }
                marker_at += 1;
                marker = *bytes.get(marker_at).ok_or(InspectError::Truncated(F))?;
            }
            let segment_at = marker_at + 1;

            match marker {
                // Markers without a length field.
                0x01 | 0xD0..=0xD7 => {
                    offset = segment_at;
                    continue;
                }
                0xD9 => {
                    return Err(InspectError::Malformed(
                        F,
                        "end of image before frame header",
                    ));
                }
                0xDA => return Err(InspectError::Malformed(F, "scan data before frame header")),
                // Baseline, extended sequential and progressive Huffman.
                0xC0..=0xC2 => return read_frame_header(bytes, segment_at),
                // Lossless, hierarchical and arithmetic-coded processes.
                0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF => {
                    return Err(InspectError::Unsupported(
                        UnsupportedKind::JpegCodingProcess,
                    ));
                }
                _ => {}
            }

            // Every other segment: 2-byte length that includes itself.
            let length = be_u16(bytes, segment_at).ok_or(InspectError::Truncated(F))?;
            if length < 2 {
                return Err(InspectError::Malformed(F, "segment length below 2"));
            }
            offset = segment_at + usize::from(length);
        }
        Err(InspectError::Malformed(
            F,
            "too many segments before frame header",
        ))
    }

    fn read_frame_header(bytes: &[u8], at: usize) -> Result<ImageInfo, InspectError> {
        // length(2) precision(1) height(2) width(2) components(1)
        let length = be_u16(bytes, at).ok_or(InspectError::Truncated(F))?;
        if length < 8 {
            return Err(InspectError::Malformed(F, "frame header too short"));
        }
        let height = be_u16(bytes, at + 3).ok_or(InspectError::Truncated(F))?;
        let width = be_u16(bytes, at + 5).ok_or(InspectError::Truncated(F))?;
        let components = *bytes.get(at + 7).ok_or(InspectError::Truncated(F))?;
        if !matches!(components, 1 | 3 | 4) {
            return Err(InspectError::Malformed(F, "unexpected component count"));
        }
        // A height of 0 means "defined later by a DNL marker", which is legal
        // but almost unused and unsupported by common decoders. It is
        // reported as zero dimensions by the caller.
        Ok(ImageInfo {
            format: F,
            width: u32::from(width),
            height: u32::from(height),
            alpha: AlphaHint::Absent,
        })
    }
}

/// WebP: <https://developers.google.com/speed/webp/docs/riff_container>.
mod webp {
    use super::*;

    const F: ImageFormat = ImageFormat::WebP;

    const VP8X_ANIMATION: u8 = 0x02;
    const VP8X_ALPHA: u8 = 0x10;

    pub(super) fn inspect(bytes: &[u8]) -> Result<ImageInfo, InspectError> {
        // RIFF header (12 bytes), then the first chunk's FourCC and size.
        let chunk = bytes.get(12..16).ok_or(InspectError::Truncated(F))?;
        let data = 20usize;
        match chunk {
            b"VP8 " => {
                // Lossy bitstream: 3-byte frame tag, start code, then
                // 14-bit width and height (top 2 bits are scaling).
                if bytes
                    .get(data + 3..data + 6)
                    .ok_or(InspectError::Truncated(F))?
                    != [0x9D, 0x01, 0x2A]
                {
                    return Err(InspectError::Malformed(F, "missing VP8 start code"));
                }
                let width = u32::from(le16(bytes, data + 6)? & 0x3FFF);
                let height = u32::from(le16(bytes, data + 8)? & 0x3FFF);
                Ok(ImageInfo {
                    format: F,
                    width,
                    height,
                    alpha: AlphaHint::Absent,
                })
            }
            b"VP8L" => {
                // Lossless bitstream: signature 0x2F, then 14 bits width-1,
                // 14 bits height-1, 1 bit "alpha is used".
                if *bytes.get(data).ok_or(InspectError::Truncated(F))? != 0x2F {
                    return Err(InspectError::Malformed(F, "missing VP8L signature"));
                }
                let bits = le_u32(bytes, data + 1).ok_or(InspectError::Truncated(F))?;
                let width = (bits & 0x3FFF) + 1;
                let height = ((bits >> 14) & 0x3FFF) + 1;
                let alpha = if (bits >> 28) & 1 == 1 {
                    AlphaHint::Present
                } else {
                    AlphaHint::Absent
                };
                Ok(ImageInfo {
                    format: F,
                    width,
                    height,
                    alpha,
                })
            }
            b"VP8X" => {
                // Extended format: flags byte, 3 reserved bytes, then 24-bit
                // canvas width-1 and height-1.
                let flags = *bytes.get(data).ok_or(InspectError::Truncated(F))?;
                if flags & VP8X_ANIMATION != 0 {
                    return Err(InspectError::Animated(F));
                }
                let width = le_u24(bytes, data + 4).ok_or(InspectError::Truncated(F))? + 1;
                let height = le_u24(bytes, data + 7).ok_or(InspectError::Truncated(F))? + 1;
                let alpha = if flags & VP8X_ALPHA != 0 {
                    AlphaHint::Present
                } else {
                    AlphaHint::Absent
                };
                Ok(ImageInfo {
                    format: F,
                    width,
                    height,
                    alpha,
                })
            }
            _ => Err(InspectError::Malformed(F, "unknown first chunk")),
        }
    }

    fn le16(bytes: &[u8], at: usize) -> Result<u16, InspectError> {
        let b = bytes.get(at..at + 2).ok_or(InspectError::Truncated(F))?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
}

/// ISO base media file format (ISO/IEC 14496-12), used by AVIF and HEIF.
///
/// Only the boxes on the path `ftyp`, `meta > iprp > ipco > ispe` are read.
/// AVIF files may declare several image sizes (alpha plane, thumbnails, grid
/// tiles); the **largest** `ispe` is reported, which is the safe choice for
/// memory limits. The decoder later confirms the primary image's real size.
mod isobmff {
    use super::*;

    const F: ImageFormat = ImageFormat::Avif;

    struct BoxHeader {
        kind: [u8; 4],
        /// Offset of the box payload.
        body: usize,
        /// Offset just past the end of the box.
        end: usize,
    }

    fn read_box(bytes: &[u8], at: usize, limit: usize) -> Result<BoxHeader, InspectError> {
        let size32 = be_u32(bytes, at).ok_or(InspectError::Truncated(F))?;
        let kind_bytes = bytes
            .get(at + 4..at + 8)
            .ok_or(InspectError::Truncated(F))?;
        let kind = [kind_bytes[0], kind_bytes[1], kind_bytes[2], kind_bytes[3]];
        let (size, header): (u64, usize) = match size32 {
            0 => ((limit - at) as u64, 8), // box extends to the end
            1 => (be_u64(bytes, at + 8).ok_or(InspectError::Truncated(F))?, 16),
            n => (u64::from(n), 8),
        };
        let size =
            usize::try_from(size).map_err(|_| InspectError::Malformed(F, "box size overflows"))?;
        if size < header {
            return Err(InspectError::Malformed(F, "box smaller than its header"));
        }
        let end = at
            .checked_add(size)
            .ok_or(InspectError::Malformed(F, "box size overflows"))?;
        if end > limit {
            return Err(InspectError::Truncated(F));
        }
        Ok(BoxHeader {
            kind,
            body: at + header,
            end,
        })
    }

    /// Finds the first child box of `kind` within `[start, end)`.
    fn find_child(
        bytes: &[u8],
        start: usize,
        end: usize,
        kind: &[u8; 4],
    ) -> Result<Option<BoxHeader>, InspectError> {
        let mut at = start;
        for _ in 0..MAX_STRUCTURES {
            if at >= end {
                return Ok(None);
            }
            let found = read_box(bytes, at, end)?;
            if &found.kind == kind {
                return Ok(Some(found));
            }
            at = found.end;
        }
        Err(InspectError::Malformed(F, "too many boxes"))
    }

    pub(super) fn inspect(bytes: &[u8]) -> Result<ImageInfo, InspectError> {
        let file_end = bytes.len();
        let ftyp = read_box(bytes, 0, file_end)?;
        if ftyp.end < ftyp.body + 8 {
            return Err(InspectError::Malformed(F, "ftyp box too short"));
        }
        // Major brand (4), minor version (4), then compatible brands.
        // Slicing is in bounds: read_box guarantees ftyp.end <= bytes.len()
        // and the check above guarantees at least 8 payload bytes.
        let (compatible, _) = bytes[ftyp.body + 8..ftyp.end].as_chunks::<4>();
        let mut brands: Vec<&[u8; 4]> = compatible.iter().collect();
        let (major, _) = bytes[ftyp.body..ftyp.body + 4].as_chunks::<4>();
        brands.extend(major);
        let has = |brand: &[u8; 4]| brands.contains(&brand);

        if has(b"avis") {
            // An AVIF image sequence. It may also contain a still "cover"
            // image, but taking only that would silently drop the animation.
            return Err(InspectError::Animated(F));
        }
        if !has(b"avif") {
            return Err(InspectError::Unsupported(UnsupportedKind::Heif));
        }

        let meta = find_child(bytes, ftyp.end, file_end, b"meta")?
            .ok_or(InspectError::Malformed(F, "no meta box"))?;
        // `meta` is a FullBox: 4 bytes of version/flags precede its children.
        let iprp = find_child(bytes, meta.body + 4, meta.end, b"iprp")?
            .ok_or(InspectError::Malformed(F, "no item properties"))?;
        let ipco = find_child(bytes, iprp.body, iprp.end, b"ipco")?
            .ok_or(InspectError::Malformed(F, "no property container"))?;

        let mut largest: Option<(u32, u32)> = None;
        let mut at = ipco.body;
        for _ in 0..MAX_STRUCTURES {
            if at >= ipco.end {
                break;
            }
            let property = read_box(bytes, at, ipco.end)?;
            if &property.kind == b"ispe" {
                // FullBox header (4), then width and height as u32.
                let width = be_u32(bytes, property.body + 4).ok_or(InspectError::Truncated(F))?;
                let height = be_u32(bytes, property.body + 8).ok_or(InspectError::Truncated(F))?;
                let pixels = u64::from(width) * u64::from(height);
                if largest.is_none_or(|(w, h)| pixels > u64::from(w) * u64::from(h)) {
                    largest = Some((width, height));
                }
            }
            at = property.end;
        }
        let (width, height) =
            largest.ok_or(InspectError::Malformed(F, "no image size property"))?;
        Ok(ImageInfo {
            format: F,
            width,
            height,
            alpha: AlphaHint::Unknown,
        })
    }
}

#[cfg(test)]
mod tests;

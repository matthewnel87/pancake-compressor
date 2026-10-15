//! EXIF and XMP handling shared by the encoders.
//!
//! **Policy (ADR-011, ADR-034):** converting encoders carry EXIF and XMP
//! where the output format can store them, so camera details, copyright and
//! captions are not silently lost. Anything that cannot be carried is
//! reported in [`crate::encode::Adjustments::metadata_dropped`]. A
//! "privacy-first: strip metadata" mode (for example to remove GPS location)
//! is a user option for M2/M3 presets.
//!
//! **Orientation:** the encoders rotate pixels upright themselves, so any
//! EXIF they write must say "upright" (orientation 1) or viewers would rotate
//! the image a second time. [`exif_upright`] rewrites that one tag in a copy
//! of the EXIF data. EXIF is untrusted input: every read is bounds-checked,
//! and if the structure cannot be followed the EXIF is not written at all
//! (and reported as dropped) rather than risk a double rotation.
//!
//! EXIF layout: optional `"Exif\0\0"` prefix, then a TIFF header (`II*\0`
//! little-endian or `MM\0*` big-endian), a 4-byte offset to IFD0, and IFD0's
//! 12-byte entries. Orientation is tag `0x0112`, type SHORT, count 1.

/// EXIF tag number for orientation.
const ORIENTATION_TAG: u16 = 0x0112;
/// TIFF field type SHORT (16-bit unsigned).
const TYPE_SHORT: u16 = 3;
/// More entries than this in IFD0 is treated as malformed.
const MAX_IFD_ENTRIES: usize = 1024;

/// Returns the EXIF data as a bare TIFF block (no `"Exif\0\0"` prefix) with
/// the orientation tag set to 1 (upright), or `None` if the data is not a
/// structure this function can safely follow.
///
/// If there is no orientation tag, the TIFF block is returned unchanged.
///
/// ```
/// use pancake_core::metadata::exif_upright;
/// // Little-endian TIFF, IFD0 at offset 8, one entry: orientation = 6.
/// let mut exif = b"Exif\0\0II*\0\x08\0\0\0\x01\0".to_vec();
/// exif.extend([0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0]);
/// exif.extend([0, 0, 0, 0]); // no next IFD
/// let fixed = exif_upright(&exif).unwrap();
/// assert!(fixed.starts_with(b"II*\0"));
/// assert_eq!(fixed[8 + 2 + 8], 1); // orientation value now 1
/// ```
pub fn exif_upright(exif: &[u8]) -> Option<Vec<u8>> {
    let tiff = exif.strip_prefix(b"Exif\0\0").unwrap_or(exif);
    let little = match tiff.get(0..4)? {
        b"II*\0" => true,
        b"MM\0*" => false,
        _ => return None,
    };
    let read_u16 = |at: usize| -> Option<u16> {
        let b = tiff.get(at..at.checked_add(2)?)?;
        Some(if little {
            u16::from_le_bytes([b[0], b[1]])
        } else {
            u16::from_be_bytes([b[0], b[1]])
        })
    };
    let read_u32 = |at: usize| -> Option<u32> {
        let b = tiff.get(at..at.checked_add(4)?)?;
        Some(if little {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        })
    };

    let ifd0 = usize::try_from(read_u32(4)?).ok()?;
    let count = usize::from(read_u16(ifd0)?);
    if count > MAX_IFD_ENTRIES {
        return None;
    }
    let mut out = tiff.to_vec();
    for index in 0..count {
        let entry = ifd0.checked_add(2)?.checked_add(index.checked_mul(12)?)?;
        if read_u16(entry)? != ORIENTATION_TAG {
            continue;
        }
        if read_u16(entry + 2)? != TYPE_SHORT || read_u32(entry + 4)? != 1 {
            // Not the standard layout; do not guess.
            return None;
        }
        // A single SHORT is stored inline in the first 2 of the 4 value bytes.
        let value_at = entry + 8;
        let upright = if little { [1, 0] } else { [0, 1] };
        out.get_mut(value_at..value_at + 2)?
            .copy_from_slice(&upright);
        return Some(out);
    }
    Some(out)
}

/// XMP packets in JPEG must fit in one APP1 segment (65,535 minus the
/// length field and the 29-byte namespace header). Larger packets need the
/// "extended XMP" scheme, which is not implemented, so they are reported as
/// dropped.
pub const JPEG_XMP_MAX: usize = 65_535 - 2 - 29;

/// Namespace header that marks an APP1 segment as XMP in JPEG.
pub const JPEG_XMP_HEADER: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

#[cfg(test)]
mod tests {
    use super::*;

    fn tiff(little: bool, orientation: u16, extra_before: bool) -> Vec<u8> {
        let u16b = |v: u16| {
            if little {
                v.to_le_bytes()
            } else {
                v.to_be_bytes()
            }
        };
        let u32b = |v: u32| {
            if little {
                v.to_le_bytes()
            } else {
                v.to_be_bytes()
            }
        };
        let mut out = if little {
            b"II*\0".to_vec()
        } else {
            b"MM\0*".to_vec()
        };
        out.extend(u32b(8));
        let entries: u16 = if extra_before { 2 } else { 1 };
        out.extend(u16b(entries));
        if extra_before {
            // ImageWidth (0x0100), SHORT, 1, value 640
            out.extend(u16b(0x0100));
            out.extend(u16b(3));
            out.extend(u32b(1));
            out.extend(u16b(640));
            out.extend([0, 0]);
        }
        out.extend(u16b(ORIENTATION_TAG));
        out.extend(u16b(TYPE_SHORT));
        out.extend(u32b(1));
        out.extend(u16b(orientation));
        out.extend([0, 0]);
        out.extend(u32b(0));
        out
    }

    fn orientation_of(tiff: &[u8], little: bool, entry: usize) -> u16 {
        let at = 8 + 2 + entry * 12 + 8;
        if little {
            u16::from_le_bytes([tiff[at], tiff[at + 1]])
        } else {
            u16::from_be_bytes([tiff[at], tiff[at + 1]])
        }
    }

    #[test]
    fn resets_orientation_in_both_byte_orders() {
        for little in [true, false] {
            let fixed = exif_upright(&tiff(little, 6, true)).unwrap_or_default();
            assert_eq!(orientation_of(&fixed, little, 1), 1);
            // The other tag is untouched.
            assert_eq!(fixed.len(), tiff(little, 6, true).len());
        }
    }

    #[test]
    fn strips_exif_prefix_and_keeps_data_without_orientation() {
        let mut no_orientation = b"Exif\0\0".to_vec();
        let mut body = tiff(true, 1, true);
        body[10..12].copy_from_slice(&0x0101u16.to_le_bytes()); // retag entry 0
        body[22..24].copy_from_slice(&0x0103u16.to_le_bytes()); // retag entry 1
        no_orientation.extend(&body);
        assert_eq!(exif_upright(&no_orientation), Some(body));
    }

    #[test]
    fn unfollowable_exif_is_refused() {
        assert_eq!(exif_upright(b""), None);
        assert_eq!(exif_upright(b"Exif\0\0XX*\0"), None);
        // IFD0 offset beyond the data.
        let mut far = tiff(true, 6, false);
        far[4..8].copy_from_slice(&5000u32.to_le_bytes());
        assert_eq!(exif_upright(&far), None);
        // Orientation stored with the wrong type.
        let mut wrong_type = tiff(true, 6, false);
        wrong_type[12..14].copy_from_slice(&4u16.to_le_bytes());
        assert_eq!(exif_upright(&wrong_type), None);
    }

    #[test]
    fn every_truncation_is_handled() {
        let full = tiff(false, 8, true);
        for end in 0..full.len() {
            let _ = exif_upright(&full[..end]);
        }
    }
}

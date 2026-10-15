//! Encoding decoded images into compressed files, then checking the result.
//!
//! Each encoder in this module returns bytes only after they have been
//! decoded again by [`crate::decode`] and compared with the source: same
//! dimensions, transparency kept, and for lossless modes identical pixels.
//! A failed check is reported as [`EncodeError::VerificationFailed`] and the
//! bytes are discarded, so a broken encoder can never produce a file the user
//! receives (`docs/17` M1: "decode then re-open every written output").
//!
//! **What it does not decide:** whether a smaller file is worth keeping, file
//! names, or where to write. Those are batch and output policies (M2).

use std::fmt;

use crate::format::ImageFormat;

pub mod png;

/// Why an encode did not produce a usable file. Stable [`EncodeError::code`]
/// values are used in progress events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// The requested options are out of range (a programming error upstream).
    InvalidOptions(&'static str),
    /// A lossy encoder could not reach the requested minimum quality. The
    /// caller should fall back (for example to lossless) rather than accept
    /// a worse image.
    QualityNotReached(ImageFormat),
    /// The source carries signed content credentials (C2PA) that re-encoding
    /// would invalidate, so it was left unchanged.
    ProtectedContent(ImageFormat),
    /// The encoder failed or ran out of memory.
    Failed(ImageFormat),
    /// The encoder produced bytes that did not decode back to the expected
    /// image. Indicates an encoder bug; the bytes were discarded.
    VerificationFailed(ImageFormat),
}

impl EncodeError {
    /// Stable machine-readable code for progress events.
    pub fn code(&self) -> &'static str {
        match self {
            EncodeError::InvalidOptions(_) => "invalid-options",
            EncodeError::QualityNotReached(_) => "quality-not-reached",
            EncodeError::ProtectedContent(_) => "protected-content",
            EncodeError::Failed(_) => "encode-failed",
            EncodeError::VerificationFailed(_) => "output-verification-failed",
        }
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::InvalidOptions(_) => {
                f.write_str("These compression settings are not valid.")
            }
            EncodeError::QualityNotReached(format) => write!(
                f,
                "A smaller {format} could not be made without dropping below the chosen quality."
            ),
            EncodeError::ProtectedContent(format) => write!(
                f,
                "This {format} carries signed content credentials, so it was left unchanged."
            ),
            EncodeError::Failed(format) => write!(f, "Creating the {format} file failed."),
            EncodeError::VerificationFailed(format) => write!(
                f,
                "The new {format} file did not pass its safety check, so it was not saved."
            ),
        }
    }
}

impl std::error::Error for EncodeError {}

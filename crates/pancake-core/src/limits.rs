//! Memory arithmetic for untrusted images.
//!
//! Image headers are attacker-controlled. A file only a few bytes long can
//! claim to be 100,000 × 100,000 pixels, which would need about 40 GB once
//! decoded. Every size estimate here therefore uses checked arithmetic: if a
//! calculation would overflow, the answer is `None` and the caller must treat
//! the file as too large rather than wrap around to a small, "safe" number.
//!
//! This module only does arithmetic. Choosing the actual budget for a batch
//! (based on available memory and parallel jobs) is M2 work, described in
//! `docs/08-batching-output-and-memory.md`.

/// Hard caps applied to every image before decoding.
///
/// These are a safety net against hostile or absurd files, not the per-batch
/// memory budget (that is M2 scheduling work). An image within these caps can
/// still be refused later if the machine lacks memory for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageLimits {
    /// Largest accepted width or height in pixels.
    pub max_side: u32,
    /// Largest accepted `width * height`.
    pub max_pixels: u64,
}

impl ImageLimits {
    /// Defaults chosen in ADR-024 (provisional until M2 memory benchmarks):
    /// 65,535 px per side covers the JPEG maximum and very wide panoramas;
    /// 2^28 pixels (about 268 megapixels, 1 GiB as RGBA8) is far beyond any
    /// camera photo while keeping a single decode within reach of an 8 GB
    /// machine.
    pub const DEFAULT: ImageLimits = ImageLimits {
        max_side: 65_535,
        max_pixels: 1 << 28,
    };

    /// Returns `true` if the declared dimensions are within both caps.
    pub fn allows(&self, width: u32, height: u32) -> bool {
        width <= self.max_side
            && height <= self.max_side
            && u64::from(width) * u64::from(height) <= self.max_pixels
    }
}

impl Default for ImageLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Bytes per pixel for 8-bit RGBA, the widest layout the engine plans to hold
/// in memory for still images in v0.1.
pub const RGBA8_BYTES_PER_PIXEL: u64 = 4;

/// Estimates the bytes needed to hold a decoded 8-bit RGBA image.
///
/// `width` and `height` come straight from an untrusted file header.
/// Returns `None` if the result does not fit in a `u64`, which callers must
/// treat as "refuse this file".
///
/// ```
/// use pancake_core::limits::estimated_rgba8_bytes;
/// assert_eq!(estimated_rgba8_bytes(1920, 1080), Some(8_294_400));
/// ```
pub fn estimated_rgba8_bytes(width: u32, height: u32) -> Option<u64> {
    u64::from(width)
        .checked_mul(u64::from(height))?
        .checked_mul(RGBA8_BYTES_PER_PIXEL)
}

/// Returns `true` only if a decoded image of the given size fits within
/// `budget_bytes`.
///
/// An estimate that overflows is never within budget. Zero-sized images are
/// reported as fitting here; rejecting them as corrupt is the format
/// sniffer's job, not the memory check's.
pub fn fits_memory_budget(width: u32, height: u32, budget_bytes: u64) -> bool {
    estimated_rgba8_bytes(width, height).is_some_and(|needed| needed <= budget_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_limits_accept_real_photos_and_refuse_bombs() {
        let limits = ImageLimits::default();
        assert!(limits.allows(8256, 5504)); // 45 MP full-frame camera
        assert!(limits.allows(16_384, 16_384)); // exactly 2^28 pixels
        assert!(!limits.allows(16_384, 16_385));
        assert!(!limits.allows(65_536, 1)); // one side too long
        assert!(!limits.allows(u32::MAX, u32::MAX));
    }

    #[test]
    fn typical_photo_size_is_exact() {
        assert_eq!(estimated_rgba8_bytes(4000, 3000), Some(48_000_000));
    }

    #[test]
    fn largest_header_values_do_not_wrap() {
        // u32::MAX squared times 4 is about 7.4e19, just above u64::MAX
        // (1.8e19), so this must report overflow, not a wrapped small value.
        assert_eq!(estimated_rgba8_bytes(u32::MAX, u32::MAX), None);
        assert!(!fits_memory_budget(u32::MAX, u32::MAX, u64::MAX));
    }

    #[test]
    fn budget_boundary_is_inclusive() {
        let exact = estimated_rgba8_bytes(100, 100).unwrap_or(u64::MAX);
        assert!(fits_memory_budget(100, 100, exact));
        assert!(!fits_memory_budget(100, 100, exact - 1));
    }
}

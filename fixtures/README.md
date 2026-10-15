# Test fixtures — provenance manifest

Every image used in tests must be **synthetic** or under a documented
redistributable licence. Never add personal photos, screenshots containing
personal data, or files whose origin is unknown (`AGENTS.md` rule 10).

| Fixture set | Location | How it is made | Licence |
|---|---|---|---|
| Header-only PNG, JPEG, WebP (lossy, lossless, extended) and AVIF/HEIF ISO-BMFF headers, including animated, truncated, oversized and malformed variants | Built in code: `crates/pancake-core/src/format/tests.rs` (builder functions at the top) | Byte-by-byte from the format specifications. Pixel data and checksums are deliberately not valid, because these tests cover header inspection only | GPL-3.0-or-later (part of this repository) |
| Random byte inputs | Generated at test time (`random_bytes_after_real_signatures_never_panic`) | Fixed-seed xorshift generator, 20,000 inputs | Not stored |

| Decodable PNG (8- and 16-bit, RGBA with transparent and half-transparent areas), lossless WebP and baseline JPEG | Generated in memory by `crates/pancake-core/src/decode/tests.rs` (`rgba_pattern`, `rgb_gradient`) using the `image` crate's encoders | Synthetic gradients; nothing stored on disk | GPL-3.0-or-later (part of this repository) |

| AVIF, 8-bit with alpha and 10-bit opaque grey; plus an edited copy whose container size is shrunk below the AV1 frame size | Generated in memory by `encode_avif` in `crates/pancake-core/src/decode/tests.rs` using `ravif` (dev-dependency) | Synthetic patterns; nothing stored on disk | GPL-3.0-or-later (part of this repository) |
| Header-only AVIF with `irot`/`imir` properties | `format/tests.rs` | Byte-built | GPL-3.0-or-later |

Encoder-specific fixtures (oxipng, imagequant, MozJPEG, libwebp lossy) arrive
with those codec steps.

Integration fixtures: `crates/pancake-core/tests/format_matrix.rs` (48×32 RGBA gradient with transparent third and half-transparent band, re-encoded by this crate into each input format) and `tests/benchmark.rs` (synthetic 4000×3000 photo-like image, generated at run time). Both GPL-3.0-or-later, nothing stored on disk.

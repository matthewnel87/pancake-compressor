# Test fixtures — provenance manifest

Every image used in tests must be **synthetic** or under a documented
redistributable licence. Never add personal photos, screenshots containing
personal data, or files whose origin is unknown (`AGENTS.md` rule 10).

| Fixture set | Location | How it is made | Licence |
|---|---|---|---|
| Header-only PNG, JPEG, WebP (lossy, lossless, extended) and AVIF/HEIF ISO-BMFF headers, including animated, truncated, oversized and malformed variants | Built in code: `crates/pancake-core/src/format/tests.rs` (builder functions at the top) | Byte-by-byte from the format specifications. Pixel data and checksums are deliberately not valid, because these tests cover header inspection only | GPL-3.0-or-later (part of this repository) |
| Random byte inputs | Generated at test time (`random_bytes_after_real_signatures_never_panic`) | Fixed-seed xorshift generator, 20,000 inputs | Not stored |

Decodable image fixtures (real pixels for codec round-trips) arrive with the
codec work in later M1 steps and will be listed here with their generator
script or source and licence.

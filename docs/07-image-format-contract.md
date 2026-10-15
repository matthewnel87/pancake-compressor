# 07 — Image format contract and accuracy policy

## Scope

**Still images only** at launch. Do not silently convert animated WebP/AVIF/GIF to a single frame. Explicitly detect animation or reject uncertain/unsupported input. The table states **planned operations**, not currently proven library support.

| Input → Output | PNG | JPEG | WebP | AVIF | Keep format |
|---|---|---|---|---|---|
| PNG | palette lossy or lossless | alpha flatten required | lossy/lossless | lossy | optimised PNG |
| JPEG | converted lossless pixels, but prior JPEG loss remains | re-encode lossy | lossy/lossless | lossy | optimised JPEG |
| still WebP | alpha possible | matte when transparent | re-encode | conversion | optimised still WebP |
| still AVIF | alpha possible | matte when transparent | conversion | re-encode | optimised still AVIF |

## Formats and exact rules

### PNG

- **Lossless:** recompression via `oxipng`; if policy is pixel-lossless, decoded pixel values including alpha must match.
- **Lossy:** `imagequant` + indexed palette/dithering, then `lodepng`; never label this visually or mathematically lossless.
- Fully transparent pixels can have nonzero RGB; compare semantically as defined in tests. Avoid halos at alpha edges.

### JPEG

- MozJPEG re-encoding is lossy even at quality 100; special lossless transforms are not included in MVP. If larger result, offer skip/keep source for keep-format path.
- Input alpha requires an explicit configurable matte, default white for web use, and a visible warning that transparency was removed.
- Orientation should be visually preserved; verify whether EXIF orientation is baked into pixels and whether output orientation tag is reset correctly (avoid double rotation).

### WebP

- Test still WebP lossy/lossless separately, alpha, color and output decodability. Animated WebP must not be silently reduced to one frame.
- Do not assume the Rust `image` crate provides general-purpose **lossy** WebP encoding.

### AVIF

- Encoder candidate `ravif`; decoder must be proven separately (`image` `avif-native` + dav1d or alternative). Test encoder native dependencies, alpha, chroma subsampling, speed and memory on macOS ARM/x64 and Windows x64.
- AVIF output may be smaller but much slower; cancel semantics must be honest. Do not suggest encoded-file bytes can be streamed without full decode by default.

## Metadata and colour management

- **No invisible metadata promise.** Map each target encoder's ability to carry ICC, EXIF, XMP, colour primaries, orientation and GPS. Test presence/absence and image appearance.
- Define two user-facing modes only **after** evidence: 'Keep colour/profile and compatible metadata where supported' and 'Privacy-first: strip metadata'. If preservation unsupported by a codec, explain and ask or skip instead of silently discarding information.
- Before-after preview should apply correct image orientation and colour-management pipeline; platform colour rendering differences must be documented.
- File name and dimension metadata is displayed locally; do not log GPS/EXIF or absolute source paths.

## Smart Auto specification v0.1

1. Inspect original format, dimensions, alpha, estimated photographic vs palette/line-art properties, requested preset and target compatibility. No remote AI.
2. In default keep-format path, compare original against optimised candidate; do not save a larger file unless explicitly requested.
3. In Web/Auto preset, attempt a **bounded** set of sensible candidates (e.g., PNG palette for simple graphics, WebP for broad support, AVIF for maximum savings if user enables it) and compare actual bytes and image quality according to preset.
4. Candidate visual similarity metrics are an enhancement, not automatically equivalent to human perception. If no dependable quality metric implemented, call the logic *rule-based auto selection*, not perceptually guaranteed optimisation.
5. If multiple files processed, report target chosen, input bytes, output bytes, and time; test counterexamples (gradient banding, text fringing, alpha edges, colour shifts).

## Suggested v0.1 presets

| Preset | Behaviour | Caveat |
|---|---|---|
| Balanced (default) | moderate visual quality and effort | format-specific tuning |
| Smaller files | stronger lossy savings | warn on artifacts |
| Best quality | conservative lossy or lossless | output may be larger |
| Lossless | only implement genuinely pixel-lossless modes for target | reject incompatible requests |
| Website | optional resize to width, preserve aspect ratio, choose web format | user-visible format choice; no upscale |

Define golden fixture pixel/visual acceptance before making claims of visual equivalence.

# Logo concepts — for maintainer selection (docs/27)

**None of these is final.** The app still uses the clearly labelled placeholder mark. A final icon needs the maintainer's choice, then a reproducible export script for `.icns`, `.ico` and PNG sizes.

`concepts-comparison.png` shows each concept at 16, 24, 32 (enlarged 4× to show pixels), 64, 128 and 256 px on light and dark backgrounds.

| Concept | File | 16 px | 32 px + | Notes |
|---|---|---|---|---|
| A — Stack | `concept-a-stack.svg` | Stripes, warm | Good | Side "squeeze" marks read as **pot handles** at small sizes. Not recommended as is. |
| **A2 — Stack, no marks** | `concept-a2-stack.svg` | Stripes, warm | **Clearly a pancake stack** | Wider, flatter middle layer carries the "squash" idea without extra marks. **Recommended** (docs/27 preferred route). |
| B — P | `concept-b-p.svg` | **Sharpest** | Good | Strong wordmark pairing, but reads as "P" more than "pancake". Dark tile almost disappears on a dark taskbar; would need a light rim. |
| C — File | `concept-c-file.svg` | Noise | OK from 64 px | Too busy small, as docs/27 predicted. Not recommended. |

## Provenance and licence

Drawn by hand as SVG source in this repository (2026-10-15), using only rectangles, ellipses and paths, with the project's colour tokens. No templates, icon packs, fonts, AI image generators or third-party artwork were used. Licence: GPL-3.0-or-later with the rest of the repository; the maintainer may relicense brand assets separately before release. A trademark search for "Pancake Compressor" has **not** been done (docs/27 legal QA).

## Rendering

The comparison sheet was rendered with a throwaway `resvg` 0.45 tool outside the repository; it is not a build dependency.

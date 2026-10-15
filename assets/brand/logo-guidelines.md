# Pancake Compressor logo

**Chosen by the maintainer on 2026-10-15: a pancake (concept A2).** Source: `logo-master.svg`.

- **Mark:** three pancakes; the wider, flatter middle layer is the "squash" (compression). Butter pat on top.
- **Colours:** cocoa edge `#8A4A1A`, pancake `#D98B3A` / `#E9A54B` / `#F0B25A`, butter `#F8D27D`, tile `#FFF4E2`.
- **Minimum size:** 16 px (reads as stacked stripes); recognisable as pancakes from 24 px.
- **Don't:** add text inside the icon, add side marks (they read as pot handles at small sizes), stretch it, or put it on busy photos.
- **In the app:** the same mark without the tile (`src/components/PancakeMark.tsx`), always next to the product name.

## Regenerating the app icons

1. Render `logo-master.svg` to a 1024 × 1024 PNG (`icon-1024.png`, committed). Any SVG renderer works; resvg 0.45 was used.
2. `npx tauri icon assets/brand/icon-1024.png -o src-tauri/icons`
3. Delete the generated `android/` and `ios/` folders (desktop app only).

Windows `.ico` and macOS `.icns` both come from step 2, so the two platforms always match.

## Provenance and licence

Hand-drawn SVG in this repository using basic shapes and project colours; no templates, icon packs, fonts or AI image generators. GPL-3.0-or-later with the repository. A trademark search for the name and mark has not been done yet (docs/27).

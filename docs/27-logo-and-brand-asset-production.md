# 27 — Logo and native application icon production specification

## Visual idea
**A stack of 2–3 simplified pancakes being lightly squashed.** It should look like an excellent contemporary productivity-app icon, not a diner logo or emoji. The squish doubles as a metaphor for file compression. Warm maple edge, butter highlight; restrained geometry; a small compression line or subtle layered negative space may convey the technical use.

## Explore three approaches before deciding
1. **Stack mark (preferred)** — 2 or 3 clean warm pancake ellipses, minimal contour, gently compressed middle layer. Best at 16–64px.
2. **P + pancake** — a simple letter P formed from pancake curves, memorable wordmark/icon pairing; risk of losing recognisability at small sizes.
3. **Folded file pancake** — tiny file silhouette stacked with a pancake layer; most literal about image compression, but may be too busy. Only use if it passes the small-size test.

## Final product language
- Main icon = simplified vector shape readable on bright/dark dock/taskbar. No text within icon. No raster textures at 16/24/32px.
- Wordmark = clear contemporary sans-serif; app name always spelt **Pancake Compressor**.
- UI may use richer pancake SVG during processing, but animation is not the logo itself. All assets local.
- The logo must be original; do not copy TinyPNG marks, other compression tools, paid emoji or unlicensed logo pack imagery.

## Deliverable checklist
- `assets/brand/logo-master.svg`: clean, editable SVG source, accessible `<title>` where embedded in content.
- `assets/brand/logo-horizontal.svg`: mark + wordmark.
- `assets/brand/logo-monochrome.svg`: true one-colour variant (works in grayscale).
- `assets/brand/logo-guidelines.md`: clear space, minimum size, colour palette, mistakes to avoid, creator/provenance and licence.
- Exports for Windows `.ico`, macOS `.icns`, PNG application bundle icon sizes and square social icon; generate via reproducible build script after design sign-off (don't hand-edit every bitmap).
- Screenshot of logo at 16, 24, 32, 64, 128, 512px on light/dark taskbar-like backgrounds.
- Provide at least 3 concept explorations and design rationale **before** finalising the production icon.

## UX art direction
Single-window, cream/light surfaces, dark cocoa text, subtle maple accents, playful SVG on empty/working/success only. Avoid generic gradients, glassmorphism overload, extraneous sidebar and trend-led ornament that slows down common actions.

## Legal/design QA
Check product name and mark availability with appropriate trademark searches before public branding; a GitHub repository name does not establish trademark clearance. Document icon/font/illustration licences; don't assume AI-generated designs are automatically free of infringement. Respect system accessibility settings and test Mac dock and Windows taskbar at small sizes.

## Acceptance
Maintain recognisability at 16px; pass monochrome and high contrast; deliver all source+native bundles; load all assets offline; no unnecessary runtime GPU costs. A placeholder is never labelled final.

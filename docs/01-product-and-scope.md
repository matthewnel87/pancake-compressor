# 01 — Product requirements and MVP scope

## Concept

**Pancake Compressor** — *Squash file sizes, not image quality.* Free and open-source desktop image compression + conversion built for designers, developers and website owners who want TinyPNG-like simplicity with 100% local processing. No paywall, sign-up, uploads, image count quotas, ads or telemetry.

## Audiences / jobs to be done

- **Web designer:** Drag a folder of hero and product images, export small website-ready assets without destroying originals.
- **Developer:** Convert mixed files to WebP/AVIF, preserve nested structure, maintain readable names and control results.
- **Photographer/creator:** Optimise still images, see honest visual differences, choose lossless when quality is critical.
- **Privacy-sensitive team:** Compress files without sending them to third parties or relying on an internet connection.

## V0.1 non-negotiables

1. Desktop builds: Windows 11 x64 NSIS EXE; Mac Apple Silicon + Intel DMG. macOS 13+ tested. Windows 10 may work but is best-effort and must not be advertised as fully supported without testing.
2. Input: PNG, JPEG, static WebP, static AVIF. BMP/TIFF as stretch only after format tests; animated WebP/AVIF/GIF never silently flattened.
3. Output: keep-format optimisation, PNG (lossy/lossless), JPEG, WebP (lossy/lossless), AVIF still images.
4. Default **Smart Auto** determines viable candidates using a documented deterministic policy. Do not promise global best compression or identical visual quality across files.
5. Batch drop/pick files and folder; no arbitrary count restriction; relative paths preserved in `_pancake` output; existing output conflicts resolved predictably.
6. Presets: Balanced, Smaller files, Best quality, Lossless (where supported), Website (resize optional); never silently upscale.
7. Queue with scanning, queued, processing, writing, completed, skipped, failed, cancelled; batch progress based on completed files, not fake codec percent.
8. Per-file old/new size and saving; total net bytes saved and error/skip count. Avoid larger output by default using skip/keep-source semantics explained in UX.
9. Accessible pancake animation while busy, screen-reader status text, reduced-motion setting.
10. Side-by-side or scrub-before/after preview at matched dimensions; never pretend thumbnails are full fidelity.
11. User-selected output destination with non-destructive defaults, cancellation and error recovery.
12. No network access by installed app; no dependencies on paid services. Open-source licence/compliance and GitHub release provenance.

## Explicitly excluded from MVP

- Cloud API/service, third-party CMS integrations, watch folders, file explorer context menu, Photoshop/Figma plugin, remote sync, telemetry, automatic updates, user accounts, mobile apps, AI cloud features, animated conversions, universal PNG/JPEG lossless transcoding claims, unrestricted multi-gigabyte image guarantee.
- Users can work with very large **files** until resource limitations apply; the system must fail safely on memory budgets. Avoid 'any 1 GB image works' claims.

## UX principles

- One focused screen: drop zone → format/preset → queue → export.
- Defaults do not require understanding encoder quality scales.
- Clarify lossy vs lossless; avoid unlabeled ambiguous 0–100 slider.
- Clear action feedback; batch stays responsive; every selected file reaches a terminal state.
- Plain-English messaging and no dismissive error codes.

## Non-functional acceptance

- Launch/offline work on both OS families; no unexpected network activity attributable to app.
- Process batches of 100+ moderate files without UI freezing or OOM.
- Cancel mid-batch, restart, hit disk-full/collision/permissions edge cases without overwriting originals.
- Loading state and per-file terminal states observable via accessible UI.
- Include install/remove and security-verification instructions on release pages.

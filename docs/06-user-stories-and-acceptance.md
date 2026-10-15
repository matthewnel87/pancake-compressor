# 06 — User stories and behaviour acceptance

**US-01 — Single-file compression:** Given a PNG, JPEG, still WebP or still AVIF in a local folder, when I drag or choose the file and press Compress, I see a processing state then a successful output or a meaningful failure. UI remains responsive.

**US-02 — Unlimited batch count (resource-bounded):** Given hundreds of files and nested folders, when I submit them, every detected supported file is queued without artificial count restriction. The app does not load every image pixel buffer at once; it reports failures individually, and keeps the queue visible/scrollable.

**US-03 — Format conversion:** Given a photo JPEG, when I choose WebP or AVIF, the resulting file has the expected magic bytes, correct dimensions and viewable output. The extension changes correctly, with no duplicated `.jpg.webp` naming unless explicitly requested.

**US-04 — Smart Auto:** Given mixed website assets, when I choose Smart Auto, documented format/preset selection rules are applied. For each file, UI reports *which* format was chosen and bytes saved. Do not claim 'best' without a measured comparison of attempted candidates.

**US-05 — Progress visibility:** Given a batch, when it runs, the pancake compresses subtly, overall `done/total` is factual, active filename/phase is shown, and successful/failed/skipped counts remain distinct. Reduced-motion disables squashing animation but retains a clear status.

**US-06 — Cancel:** Given a running batch, when Cancel is pressed, queued jobs cancel quickly and active jobs stop at the nearest safe cancellation point. Existing completed outputs remain, temporary files are cleaned and terminal counters reconcile.

**US-07 — Recoverable errors:** Given a corrupt image, readonly destination, filename collision or insufficient memory, that file fails or skips with a clear cause; other files continue if safe. The app never overwrites source files by default.

**US-08 — Destination preservation:** Given `assets/products/a.jpg`, when I export into `_pancake`, the result is `assets/_pancake/products/a.webp` or under explicitly chosen destination with preserved relative tree. Files inside `_pancake` are not picked up on rescans. Path traversal attempts are rejected.

**US-09 — Privacy:** Given no network connection, when I launch, select, preview, compress and export, all operations succeed with no app-originated network activity. No authentication or advertising surfaces appear.

**US-10 — Image fidelity:** Given EXIF orientation, alpha and ICC fixtures, output matches format contract. An animated image is not silently truncated to its first frame; unsupported animation gets a clear error/skipped state.

**US-11 — Quality choice:** Given a preset, UI explains trade-offs and quality controls if applicable. Lossless means exact pixel value preservation (where implemented, colour/metadata constraints clear), not that a lossy codec at `quality=100` is lossless.

**US-12 — Preview:** Given a successful output, visual compare uses same orientation, crop, zoom and display transform on both sides. Preview thumbnails are bounded to avoid OOM and disclose scaled viewing.

**US-13 — Installer:** Given release packages, on Windows x64 and both macOS architectures I can install, launch, process a sample image completely offline, find exported files, exit and uninstall. Release metadata specifies verified OS/arch, signature status, checksum and provenance.

**US-14 — Security verification:** Given a downloaded official release, I can compare SHA256, use GitHub attestations and (if signed) OS signature verification. Documentation includes precise commands and limitations.

## Acceptance format

Each story requires (1) automated test ID, (2) manual UX reproduction steps, (3) screenshot/log without private user content, (4) platform evidence, and (5) failure-condition coverage. Track these in `docs/IMPLEMENTATION-STATUS.md` after starting development.

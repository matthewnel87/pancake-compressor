# 12 — Visual design, accessibility and interaction system

## Brand / appearance

Pancake Compressor should feel clean and trustworthy, not gimmicky. Warm off-white surfaces, restrained honey/amber accent, crisp readable typography, high-contrast text and simple pancake icon. Support system light/dark theme. Avoid depending on remote font servers. No intrusive splash screens or login.

## Single-window IA

1. Top bar: Pancake icon/name; '100% local' small factual indicator; settings, help.
2. Main drag area with accessible 'Choose images' and 'Choose folder' controls; drop feedback.
3. Controls: Output format `Auto/Keep/PNG/JPEG/WebP/AVIF`; preset `Balanced/Small/Best/Lossless/Website`; optional advanced metadata/resize settings.
4. Process queue: filename, state, size-before/after, savings, failure reason, optional preview.
5. Persistent batch status: `N of M processed`; determinate progress; current active files; Cancel.
6. Final summary: optimised / unchanged / skipped / failed; net saved bytes; Open output folder.

## Keyboard and screen-reader requirements

- All actions reachable by Tab and Shift+Tab; visible focus ring and semantic button/select elements.
- Drop zone has equivalent button controls. Do not require dragging.
- Progress has `role=progressbar`, `aria-valuenow`, `aria-valuemax`, `aria-valuemin`, descriptive `aria-label`, and associated textual processed count.
- Screen readers receive throttled updates at meaningful boundaries, not every frame; `aria-live="polite"` for summary, `role="alert"` for destructive/critical failures only.
- Status is not colour-only: text and icons accompany success/warning/failure; result percentage is not the only meaning.
- UI scales up to 200% text; supports narrow/small desktop window and high-DPI displays.
- Ensure WCAG 2.2 AA contrast goals and native platform conventions where practical.

## Motion spec

Pancake icon has lightweight compression transform while any task is active; 1–1.5s easing loop. **Animation is not actual completion percentage.** Actual progressbar is data-driven and uses no fake time-based tween that suggests work progressed before a file finishes. Under `prefers-reduced-motion: reduce`, disable animation and show the same status statically.

## Error copy / UX text

| Condition | User-facing wording |
|---|---|
| Loading selected files | `Checking images…` |
| Batch active | `Squashing your images… 14 of 40 processed` |
| Large file limited | `This image needs more memory than the current limit. Try Low-memory mode or a smaller image.` |
| Animated image | `This animated format isn't supported yet. Your original was not changed.` |
| Format won't support transparency | `JPEG can't keep transparency. Choose a background colour or another format.` |
| No size savings | `Already compact — original kept.` |
| Cancel | `Stopped. Completed images are saved; unfinished images were not exported.` |
| Destination no permission | `Can't write to this folder. Choose another output location.` |

## UI acceptance

Test a full batch while zoomed at 200%, keyboard-only and with reduced motion. Start, cancel, format change and export must remain discoverable and usable without colour distinctions. All error messaging should identify what happened and how to fix it.

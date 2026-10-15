# 23 — UI/UX, motion, and speed contract

## Primary audience and jobs to be done
A designer, website owner or developer wants to prepare 1–1,000 local images for a website with **minimal decisions**. Users must be able to start without registration, tutorials, format jargon or visits to a help website. There is one window, a default Smart Auto setting and an obvious primary action.

## The one-window interaction

```text
┌ Pancake Compressor 🥞                        [Theme] [?] [Settings] ┐
│  Your images, smaller. All on your computer.                     │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │ Drop images or folders here     [Choose images] [Folder]  │   │
│  └────────────────────────────────────────────────────────────┘   │
│  [Smart Auto ▾] [Keep original format ▾] [Export: _pancake ▾]     │
│  12 images · 48.6 MB                                            │
│  filename              status               original → output   │
│  hero.png              Done                 4.2 MB → 980 KB     │
│  feature.webp          Compressing          Working…            │
│  banner.jpg            Queued               —                   │
│  [real progress 4/12]  [Cancel] [When finished: Open folder]     │
└──────────────────────────────────────────────────────────────────┘
```

## States and behaviours
1. **First launch**: uncluttered hero/drop zone; 'Local. Private. Free.' tiny line; keyboard alternatives for drag/drop; do not show a wizard.
2. **Queue**: accept file/folder, count valid files, show parsing state for large directories, reject malformed/symlink cases with explanation. Default outputs to adjacent `_pancake` directory; never overwrite originals.
3. **Processing**: batch progress = terminal file count ÷ accepted job count. File phase can be decoding, encoding, writing; if decoder lacks true percentage show indeterminate state for that file. Throttle UI events while guaranteeing final events.
4. **Completed**: honest totals (optimised/unchanged/skipped/failed), bytes saved, open folder, retry failed. Negative savings handled as 'no improvement' and original preserved. Distinguish output selection from conversion failures.
5. **Preview**: optional side-by-side matched zoom; thumbnails generated boundedly, locally and asynchronously. Do not decode megapixel images on React thread or send huge RGBA arrays over Tauri IPC.
6. **Errors**: message what happened + action (e.g. choose a writable folder); preserve other batch results.

## Motion
- Original pancake SVG layers squish vertically using `transform`/`opacity`; no layout reflow. Decoration only, not a proxy for progress.
- Motion duration about 0.9–1.5s, max one looping animated element in visible processing section. Stop loop on completion/cancel.
- `prefers-reduced-motion: reduce` removes decorative movement while progress counts and text remain.
- No animation package unless performance proof requires it; CSS transitions and light SVG preferred.

## Proposed performance budgets (targets to measure; not promises)
- Fresh startup into usable empty screen: **target ≤2 seconds** on a defined typical Win/macOS test machine; publish hardware and cold/warm measures. No claim until measured.
- Dragging a few images: visible queue feedback **target ≤150ms** (subsequent scanning is asynchronous).
- UI interactions during heavy compression: **target ≤100ms** perceived response; no JS main-thread decoding.
- React view should virtualise or window large file tables and avoid per-file re-render storms (test a 1,000-item queue).
- Cap simultaneous thumbnail work and memory; use event aggregation (~5–10 UI updates/sec plus guaranteed terminal events).
- Compare compressed file bytes and quality against a reproducible fixture corpus; avoid claiming universal superiority over TinyPNG/Caesium.
- Tiny animations must not block core Rust work. Audit installer/app size, idle RAM, peak RAM, startup and job throughput on actual supported hardware.

## Accessibility gates
Full keyboard-only task completion, high contrast, visible focus, screen-reader announcements at semantic milestones, 200% text, scalable Windows display settings, dark mode, no colour-only states, accessible compare slider, no audio dependence. See `docs/12-accessibility-and-design-system.md`.

## UX acceptance walkthrough
A new user must: (a) select 10 mixed images, (b) understand Smart Auto, (c) see honest progression and current file, (d) export without damaging originals, (e) open results, (f) see exact aggregate savings, (g) report an error if one file fails. Observe 3–5 participants on both OS families, capture friction anonymously only with consent, and turn findings into issues; no silent telemetry.

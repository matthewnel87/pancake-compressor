# 02 — Architecture decisions / ADR ledger

Decisions marked **provisional** require M0/M1 evidence; do not treat guesses as established requirements.

| ID | Decision | Reason | Status |
|---|---|---|---|
| ADR-001 | Product name Pancake Compressor | User-selected brand | Final |
| ADR-002 | Tauri v2 + React/TypeScript/Vite/Tailwind | Thin native desktop, rich UX, Rust IPC | Final |
| ADR-003 | Rust `pancake-core` crate separate from desktop | Test codecs without GUI; future CLI possible | Final |
| ADR-004 | No runtime networking or updater in MVP | Privacy/security/reproducibility | Final |
| ADR-005 | GPL-3.0-or-later anticipated | `imagequant` GPL licensing compatibility; audit before publish | Provisional pending audit |
| ADR-006 | Output `_pancake/`; never overwrite originals by default | Data loss prevention | Final |
| ADR-007 | Windows x64 NSIS + macOS ARM64/Intel DMG | Distributable desktop apps | Final |
| ADR-008 | OS versions: Windows 11 x64, macOS 13+ | Testable support window | Provisional validation |
| ADR-009 | Smart Auto rule-based; no cloud model | Predictable, zero cost, local | Final |
| ADR-010 | Still images only in v0.1 | Prevent animation loss | Final |
| ADR-011 | No implicit metadata discard; clear policy in format spec | Privacy and visual fidelity | Final |
| ADR-012 | Public-repo GitHub Actions standard runners | Free build and test automation | Final |
| ADR-013 | Unsigned preview is clearly marked; public stable Mac release ideally signed/notarised | Respect OS security | Final policy, funding unresolved |
| ADR-014 | Use native OS dialogs and Rust file operations; avoid broad FS plugin | Reduce attack surface | Final |
| ADR-015 | Default 'avoid bigger outputs' policy | Avoid regression but report skipped files | Final |

## Decisions left for codec spike (do not guess)

1. **Exact encoder binding and versions** across Win x64, macOS x64/ARM64; native library linking or vendoring, patch licensing.
2. **AVIF input decoder**: `image` crate with `avif-native` feature + dav1d or alternate audited decoder; packaging has to pass on all platforms.
3. **Metadata**: which encoders can preserve ICC/EXIF/XMP natively? Where unsupported, clearly warn or block sensitive modes. Establish truthful default.
4. **Color correctness**: profile conversion/embedding decisions; perform golden-image comparisons (not merely raw RGB delta).
5. **Smart Auto**: compressed candidates and scoring. Start with bounded candidates; benchmark speed/time/memory.
6. **Mac app signing**: Apple Developer Program funding or preview-only distribution with user-facing explanation. No hidden bypasses.
7. **Windows code signing**: evaluate SignPath Foundation after public release or paid certificate; no approval assumed.

## Change control

For every new decision: create `ADR-NNN` entry with date, options considered, security/licensing consequence, tests, and reason. Do not silently change output semantics, permissions or supported OSes.

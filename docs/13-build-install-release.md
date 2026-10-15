# 13 — Build and install: Windows and macOS

This document is the **installer deliverable contract**. The handoff currently contains no executable app; all builds must be performed by Codex/on GitHub after scaffolding.

## Target build matrix

| Target | GitHub standard runner | Rust target | Bundle | Planned filename |
|---|---|---|---|---|
| Windows 11 x64 | `windows-latest` (ensure x64) | `x86_64-pc-windows-msvc` | NSIS `.exe` | `Pancake-Compressor_vX.Y.Z_windows_x64-setup.exe` |
| macOS 13+ ARM | `macos-15` (Apple Silicon) | `aarch64-apple-darwin` | `.dmg` | `Pancake-Compressor_vX.Y.Z_macos_arm64.dmg` |
| macOS 13+ Intel | `macos-15-intel` | `x86_64-apple-darwin` | `.dmg` | `Pancake-Compressor_vX.Y.Z_macos_x64.dmg` |

Runner labels and available hardware evolve. Confirm against GitHub's runner documentation when first executed; use alternative listed standard runners if needed. OS version is a **support target to test**, not something merely determined by build-runner OS.

## Build dependencies

- Windows: Node LTS, Rust stable MSVC toolchain, Microsoft C++ Build Tools and WebView2. For **offline installation** configure `bundle.windows.webviewInstallMode.type = "offlineInstaller"` in `src-tauri/tauri.conf.json`; this increases installer size by roughly 127 MB but avoids needing a WebView2 download at install time. Benchmark and document the actual size.
- Mac: Node LTS, Rust stable, Xcode Command Line Tools and appropriate Rust target; build DMG on macOS host. Native codec bindings must link for target architecture.
- `npm ci` + Cargo lockfile and native dependencies; never ship external asset/font CDN references.

## Configuration requirements

- `tauri.conf.json`: `productName: Pancake Compressor`, appropriately owned reverse-domain `identifier`, version matching manifest and tag, bundle targets built per OS with CLI, app icon for Win and Mac, CSP and only exact enabled capabilities. Disable updater in MVP. For Mac unsigned **preview** builds use `APPLE_SIGNING_IDENTITY=-` for ad-hoc signing, knowing this does not bypass Gatekeeper. Stable macOS distribution needs Developer ID and notarisation.
- Ensure backend binary and native codec dylibs/resources are included and load offline. Prefer static linkage where legal/appropriate and test on clean devices.
- Windows NSIS installer should present recognisable app name/publisher info if signed; provide uninstall entry, install to correct per-user location and verify uninstaller removes app **without deleting user exported images**.
- macOS DMG should support drag app into Applications; separately build per architecture, or a universal bundle only if actually tested. `.dmg` ≠ `.pkg`.

## Manual proof protocol, every release candidate

**Windows x64**

1. Download canonical GitHub release EXE; compare SHA256 and attestation.
2. Verify signature status; if unsigned preview, document SmartScreen warning accurately—never promise no warning.
3. Install on clean Windows 11 x64 system with WebView2; confirm installer/uninstall entry.
4. Launch without internet; import 3 formats, compress PNG→WebP, JPEG→AVIF and keep-format; verify output opens correctly.
5. Batch cancellation, folder nesting, alpha and corrupt-file error. Reboot, reopen, uninstall, check exports untouched.

**Mac ARM64** and **Mac Intel** independently

1. Download matching `.dmg`, SHA256 and attestation check.
2. If signed/notarised, run `spctl --assess --type execute --verbose /Applications/Pancake\ Compressor.app` and `codesign --verify --deep --strict --verbose=2 /Applications/Pancake\ Compressor.app` after install, inspect/notarisation ticket as appropriate.
3. Mount DMG, drag `.app` into Applications, eject; verify launch from Applications. Do not direct normal users to blanket-disable Gatekeeper.
4. Offline PNG/JPEG/WebP/AVIF processing; output quality/metadata checks; cancel/error; reboot/relaunch and trash/uninstall.
5. For **unsigned previews**, clearly state Gatekeeper may block launch and prefer a signed/notarised stable release for nontechnical users.

## Workflow expectation

`.github/workflows/ci.yml` runs lint/unit/test; `.github/workflows/release.yml` builds **actual Windows EXE plus two DMGs** from a `vX.Y.Z` tag, generates SHA256 and attempts GitHub provenance attestations. The workflow publishes a **pre-release** by default until manually vetted/signed; never equate a GitHub Actions success with end-user installation success.

## Release acceptance

Stable public release requires installers on all three targets, platform smoke evidence, vulnerability/licence gate, no-network runtime test, release checksums and provenance, truthful signing/notarisation disclosure, copy-tested installation docs. If Mac signing isn't funded yet, release as **unsigned technical preview** only and make the limitation conspicuous.

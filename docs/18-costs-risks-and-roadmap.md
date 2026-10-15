# 18 — Costs, risk register and future roadmap

## Cost and distribution policy

| Item | Required | Price / caveat |
|---|---|---|
| Tauri, Rust, React, Vite, Tailwind | Yes | Free and open source |
| Image codecs | Yes | Free subject to open-source licence obligations; `imagequant` GPL constraints |
| GitHub public source and standard CI runners | Yes | Available free on public repositories; observe storage/retention and product changes |
| Download hosting | Yes | GitHub Releases; repo owner monitors policies/limits |
| macOS Developer ID signing + Apple notarisation | Consumer-friendly macOS launch | Typical Apple Developer Program US$99/year; requires Apple identity and credentials |
| Windows signing | Trust/identity | SignPath Foundation free *if eligible and approved*, otherwise potentially paid; new signatures can still encounter SmartScreen |
| Web/backend/API | No | £0; app runtime deliberately has no internet dependencies |
| Third-party security audit | Optional future | Not necessary to begin but don't claim independent audit without one |

**Critical offline-install choice:** Tauri Windows NSIS default WebView2 `downloadBootstrapper` can require internet at installation if WebView2 is absent. To guarantee offline installation, set `bundle.windows.webviewInstallMode.type = 'offlineInstaller'`, accepting an increase around 127 MB in installer size, or explicitly restrict release guarantee to *offline after installation*. The project priority is **offline app AND offline installer**, so configure `offlineInstaller` for published Windows installers. Confirm fresh Windows clean install works and test install size. Source: https://v2.tauri.app/distribute/windows-installer/

## Risk register

| Risk | Likelihood/impact | Mitigation | Owner/stage |
|---|---|---|---|
| Decoder exploited by malicious image | Med / critical | Audit, fuzz, dimensions/memory limit, isolation where needed | M1/M4 |
| AVIF native dependency breaks one architecture | Med / high | Compile spike first on three targets, fallback codec ADR | M1 |
| Bad colour/EXIF transforms | High / high | Golden tests, explicit metadata policy and conservative fallback | M1 |
| Memory exhaustion from big image | High / high | Bound jobs + decoded pixel budget; safe failure | M2/M4 |
| Files overwritten or symlink escape | Med / critical | Deny symlinks, registry of allowed roots, temporary atomic output, tests | M2/M4 |
| User believes 'unlimited' means unlimited hardware | High / medium | Explain count unmetered, resources bounded | Product/UI |
| Unsigned Mac download blocked by Gatekeeper | High / high | Preview caveat; get Developer ID/notarise for stable | M5/M6 |
| Unsigned Win download causes warnings | High / medium | Honest warnings, hashes, attestations; SignPath application | M5/M6 |
| SignPath approval denied | Med / medium | Optional paid signing or clearly preview state | M6 |
| Packaging dependencies not self-contained | Med / high | Clean device offline smoke tests | M5 |
| GPL/non-compliant transitive package | Med / high | cargo-deny, audit, licence notices, legal review | M1/M4 |
| UI animation reports bogus progress | High / medium | Measured file-state progress, distinct indeterminate animation | M3 |
| CI artifact mistaken for validated installer | Med / critical | Draft release, manual OS installation gate | M5 |
| Unmaintained software has known CVEs | Med / critical | Dependabot, maintenance policy, security advisories | Ongoing |

## Product roadmap after first release

**v1.1**: more website-specific presets (hero image, product photography, icons), resize presets, drag-out results, quick export to any folder, stronger performance reports, context menu integration optional.

**v1.2**: optional target-size budget, `srcset` multi-resolution exports, deterministic CLI for local build scripts, improved folder presets and keyboard-first usability.

**Later**: Linux releases, Windows ARM64, HEIC import where legally/technically feasible, plugin/automation integration *only if users request it and security model permits*. **Never** add cloud uploads or telemetry as a default or paywalls.

## Success metrics (measured locally in test corpus, not telemetry)

- Reliable functional installs and uninstalls on each advertised target.
- Zero source corruption in adversarial, interrupted and permission-denied tests.
- Documented batch throughput and peak RAM per hardware tier and format.
- Per-format representative compression effectiveness / visual-quality metrics vs source, ImageOptim, Caesium and TinyPNG at declared settings.
- Time to first useful output with user tests. Accessible keyboard-only completion of top workflow.

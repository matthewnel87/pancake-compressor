# 10 — Threat model and required security controls

## Assets / boundaries

Assets: user's confidential image contents and metadata, originals, saved results, filesystem integrity, application binaries/signing identity, source repository, CI token/secrets. Boundaries: file → native decoder, untrusted webview → Rust IPC, Rust app → user disk, source/dependencies → CI runner → release downloads.

| Threat | Example | Required mitigation | Evidence |
|---|---|---|---|
| Image parser exploit | Malformed AVIF exploits decoder | Patch dependencies, minimise native codecs, strict input validation, fuzz selected parser paths; worker-process isolation investigation | Dependency scans, fuzz logs, corpus |
| Decompression bomb | Huge dimensions in small file | Preflight dimension/pixel/memory checks, bounded concurrency | Resource-limit tests |
| Source file damage | Collision or partial overwrite | Default no-clobber, temp files, explicit destination, safe finalize | Crash/cancel/collision tests |
| Symlink/path traversal | Nested folder escapes output root | No follow, secure root confinement, reject out-of-root destinations, TOCTOU review | Platform security tests |
| UI injection | Crafted filename rendered as HTML | React escaping, CSP, no remote content, validate all IPC paths | Security UI tests / config review |
| Arbitrary file access | Compromised webview calls broad Tauri plugin | Minimum Tauri capabilities, no shell/http/global fs, backend scoped tokens | Config audit + adversarial IPC tests |
| Supply chain poisoning | Compromised dependency or CI action | Lockfiles, dependency scanners, provenance, minimal permissions, code review, pinned actions at tested commit when possible | CI workflow and audit report |
| Release swap | Attacker uploads modified installer | SHA-256, GitHub attestation, signed binaries if funded/approved | Verification walkthrough |
| Leakage through logs | Absolute paths, EXIF GPS | Minimal logging, redact paths, no uploads | Source review and traffic test |
| 'Local-only' false claim | Telemetry updater or remote UI asset | No network plugin/CDNs, offline smoke test, OS firewall/packet capture | Reproducible no-network test |

## Defence in depth

1. **Preflight** files by magic bytes + declared dimensions/animation/memory requirements; reject unsupported encodings and unpredictable metadata paths.
2. **Use bounded resources**, not a naive `read_to_end` + `par_iter` on hundreds of images.
3. **Keep OS access minimal**. The Tauri capability system restricts UI access to plugin commands, **not all privileged Rust code**; backend validation is mandatory.
4. **No remote content**. Bundle JS/CSS/fonts/icons locally, use strict CSP (no `unsafe-eval`, unsafe inline as feasible), disable devtools in release if appropriate, do not expose custom protocol serving arbitrary filesystem paths.
5. **No shell code execution** from UI; no unsafe dynamic libraries downloaded at runtime.
6. **Review native codecs**. Rust memory safety does not make foreign C libraries memory-safe. Consider isolated unprivileged worker processes for untrusted decoders before stable v1 if threat testing warrants it.
7. **Harden release pipeline**. Workflow GITHUB_TOKEN least privilege, release builds from tags, no release secrets on PRs, no unreviewed pull request builds with write/secrets, attestation for binaries.
8. **No claims beyond proof**. Public docs separate source availability, reproducibility, artifact attestation, OS signing and malware scanning. They are different properties.

## Minimum security gate before public preview

- Rust/JS dependency audit clear of known high-severity unfixed vulnerabilities (any waivers documented with risk and expiry); source licence audit.
- Tauri capability/CSP review; no network or shell plugin; tests for rejected malicious IPC input.
- Fuzz/integration tests on corrupt images and pixel bombs; no silent animation truncation.
- Path traversal/symlink/collision/disk-full tests on macOS **and** Windows.
- Installer checksums and attestation for each artifact; transparent signing status.
- Offline functional test with blocked outgoing connectivity.

## Security reporting

Use `SECURITY.md`; enable GitHub private vulnerability reporting after repo creation, do not include a fabricated security email. Track release/security issues without private exploit details.

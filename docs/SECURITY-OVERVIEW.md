# Security & privacy — read this before installing

> **Status:** The repository handoff is a plan; no released binaries or completed runtime security tests have been verified in this package. Re-evaluate and update this page for every published version.

## What Pancake Compressor is designed to do
- Process images locally, without uploading them to a service.
- Require no login, subscription, trackers or runtime analytics.
- Keep original images intact by default, exporting to a separate folder.
- Accept only supported local still-image formats and reject malformed/high-risk inputs safely.
- Use small desktop permissions and controlled Rust filesystem operations.

These are **implementation requirements**, not claims that an unreleased application already passes them. Inspect `docs/IMPLEMENTATION-STATUS.md` for actual evidence.

## How downloads should become verifiable
Official binaries will live in the project's GitHub Releases once independently tested. Each release should link to the exact source commit, SHA-256 checksums, GitHub Actions provenance, signing/notarisation status and a clean-install test record. Build provenance is not a malware-free guarantee.

## Your computer may show OS warnings
Unsigned Windows EXEs can trigger SmartScreen. Unsigned/non-notarised macOS DMGs can be blocked by Gatekeeper. We will disclose this honestly. **Do not disable system-wide protections to install Pancake Compressor.** Prefer signed, verified distribution; development previews may be for experienced testers only.

## Report concerns
Use private vulnerability reporting in the repository's **Security** section (requires maintainers to enable it). Never disclose exploit details, private images, local file paths or security-sensitive traces publicly. General feedback belongs in GitHub Discussions; bugs in Issues.

Read [`SECURITY.md`](../SECURITY.md), [`PRIVACY.md`](../PRIVACY.md) and [`docs/28-security-trust-and-public-evidence.md`](28-security-trust-and-public-evidence.md) for details.

# 21 — Public trust centre: what users can actually verify

Create a GitHub documentation page called **Safety, Privacy & Verification** when first downloadable builds exist. Use evidence, not empty badges or '100% secure' marketing.

## Proposed public page

### Where do my images go?

Pancake Compressor processes images locally. It does not upload your images, require an account, contact an API or send telemetry. [Link to exact privacy policy and offline test evidence per version.]

### What protects my files?

The app defaults to a new output directory, does not overwrite originals, checks image formats and dimensions, limits simultaneous memory use, and uses safe temporary files. [Link to tested version, tests, known limitations.]

### How was this download built?

All source code is public under GPL-3.0-or-later. Official downloadable installers are built from tagged commits on GitHub Actions. Each downloadable asset has a SHA-256 digest and GitHub build attestation; verify the actual download via GitHub CLI. [Link to GitHub release + CI run + tag + checksums.]

### Is the app signed?

Provide an **OS-by-OS table** for each release. An unsigned Windows EXE may display SmartScreen warnings; unsigned Mac DMGs can trigger Gatekeeper blocks. Do not badge an unsigned build as 'verified publisher'. [Link to actual Developer ID / SignPath identity and check commands only when live.]

### What security testing is performed?

Public CI covers Rust and TypeScript lint/tests, known vulnerability checks and malformed-file tests. Manual release smoke tests should cover clean installs on Windows x64 and Mac ARM/Intel, offline operation, malicious paths, corrupt images and cancellation. [Link to test log/evidence; specify which tests were not performed.]

### Can open-source software still be unsafe?

Yes. Open source permits independent review but does not eliminate mistakes. Signed code guarantees publisher identity, not safety. GitHub attestations prove build provenance, not freedom from vulnerabilities. Invite responsible security disclosure through GitHub's private vulnerability reporting.

## Proof bundle to publish with each version

- App version, source tag, exact commit and release asset names.
- SHA256SUMS.txt plus platform-specific verification commands.
- GitHub attestation result or exact verification URL.
- Windows signature state and issuer if signed.
- Mac ARM and Intel code-signing and notarisation state.
- Test evidence by OS, including app launch and no-network check.
- Outstanding security warnings, vulnerable-dependency waivers and known image compatibility limits.

## Explicit prohibition

Do not issue a 'security certified', 'virus free', 'fully private' or '100% safe' badge without a real independently scoped assessment. Security claims are version-specific, testable and revocable.

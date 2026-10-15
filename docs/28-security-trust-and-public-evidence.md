# 28 — Security information that earns trust

## Make claims with evidence, not badges
- "Offline" means the installed app works with connectivity disabled and has no runtime image upload/telemetry features; document tested builds and OS versions. Don't imply GitHub build/update traffic is impossible.
- "Safe" cannot be proven absolutely. Document security design, protections and known limitations honestly.
- "Signed/notarised" must match actual binary signatures; SHA-256 and GitHub attestations verify provenance/integrity but do not prove malware-free operation.

## Required technical controls
1. Treat image formats as hostile bytes: validate signatures, dimensions, pixel count, animation/metadata policies and total working-memory estimates before allocation.
2. Keep file IO behind narrow Rust/Tauri commands; user-authorised paths only; no unbounded filesystem permissions or remote content.
3. No user content to logs, telemetry, network, clipboard syncing or temp folder leaks. Redact absolute paths in crash/report text.
4. Bounded queue, cancellation and atomic safe output; originals preserved; block traversal/symlink escapes.
5. Supply-chain safety: lockfiles, reviewed dependency changes, cargo audit/deny, npm audit, licence reports, security alerts and third-party notices.
6. CI/build integrity: least GitHub Actions permissions, pinned actions where feasible, protected release branches, native build matrix, SHA-256 and build attestations, manual verified installation. Enable immutable releases where available.
7. No invisible updates, remote scripts or runtime analytics in first release.

## Public security publication set
- `SECURITY.md`: supported versions, private vulnerability reporting through GitHub Security advisories (enable private reporting in repo settings), disclosure process and expected response practices.
- `PRIVACY.md`: local file handling, no tracking, optional logs/redaction, absence of network runtime behaviour as implemented.
- `docs/SECURITY-OVERVIEW.md`: nontechnical summary; differentiates policy, implemented controls and independent verification.
- `docs/INSTALL-WINDOWS.md` and `docs/INSTALL-MACOS.md`: install/OS permission cautions, precise signing status, hashes and provenance verification once generated.
- `docs/14-signing-and-provenance.md`: procedure and exact evidence links for each release.
- `docs/IMPLEMENTATION-STATUS.md`: do not mark security tests as passed until output is recorded.

## Release trust manifest (generated only once binaries exist)
For each release: version/tag/commit, file name, architecture, bytes and SHA-256, GitHub build/workflow URL and attestation, signature identity/status, notarisation status, run-based AV/dependency audit information with date/scope, actual test OS/hardware, limitations and fixed vulnerabilities. Report failures explicitly. Publish SBOM where feasible.

## Privacy and vulnerability feedback separation
Ordinary users leave reviews and suggestions in Discussions. Reproducible bugs use Issues with sanitised files/logs. **Vulnerabilities are reported privately**, not via public issue template. Maintain a non-public security disclosure channel before encouraging reports. Never send photo files automatically.

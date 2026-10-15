# Security Policy — Pancake Compressor

> **Pre-release specification.** Controls are planned until implemented, tested and linked to a release. Do not present this document as evidence that an as-yet-unbuilt binary is safe.

## Supported versions

A version becomes supported only after public release. Keep the most recent stable release supported, document critical-fix policy in release notes, and never promise SLA response times without a maintained team.

## Reporting a vulnerability

Use GitHub **Private Vulnerability Reporting** once enabled in the public repository: Security → Advisories → Report a vulnerability. Do **not** post exploit payloads, private image files or undisclosed security weaknesses as public issues. Before enabling that feature, document a maintainer-controlled security contact without publishing private addresses in generated files. Acknowledge reports when practicable, triage impact, coordinate fixes and publish advisories for confirmed issues.

## Threat model summary

Untrusted image payloads can target codecs or cause decompression bombs; crafted paths can overwrite/unintentionally expose files; UI compromise could invoke commands; supply-chain compromise can alter code, dependencies or release artifacts. Primary mitigations: validation, memory budgets, bounded concurrency, path confinement, non-destructive outputs, narrowly scoped IPC, dependency pinning and audits, CI provenance and installer signing where available. See `docs/10-threat-model.md`.

## Privacy promise

**Files remain on-device.** No runtime network endpoints, analytics, background uploads, embedded remote content or updater in the MVP. We do not claim an OS's own diagnostics/security checks are controlled by the app. See `PRIVACY.md`.

## Verifying downloads

Publish SHA-256 digest(s) and GitHub-generated build provenance for **each** release binary, together with the exact public commit/tag and OS-specific signing status. Example only, after repository exists:

```bash
# macOS — locally compute and compare against published SHA256SUMS.txt
shasum -a 256 Pancake-Compressor-*.dmg
# With GitHub CLI and the actual public repository path:
gh attestation verify ./Pancake-Compressor-<version>-<platform>.dmg --repo OWNER/pancake-compressor
```

```powershell
# Windows PowerShell
Get-FileHash .\Pancake-Compressor-Setup.exe -Algorithm SHA256
```

Hash matches prove bytes match the published checksum; signatures tie artifacts to signing identities; attestations tie them to specific workflows/source. **None alone certifies the program is vulnerability-free.** Always install only from the canonical GitHub Releases page, not mirrors.

## Release signing policy

See `docs/14-signing-and-provenance.md`. Do not mark unsigned preview builds as signed. For Windows, investigate SignPath Foundation eligibility after a maintained public release; for macOS, conventional Developer ID signing and notarisation need a paid Apple Developer Program membership. Store signing secrets only in protected GitHub environments.

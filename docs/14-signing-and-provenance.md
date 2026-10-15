# 14 — Release authenticity: hashes, attestations, signing and notarisation

## Different assurances (do not conflate)

- **SHA-256** proves downloaded bytes match a displayed digest; publishing a hash on the same compromised page doesn't independently establish trust.
- **GitHub artifact attestation** connects artifact digest to the GitHub Actions build workflow, repository and commit/tag. It **does not** certify absence of malware/vulnerabilities.
- **Code signing** identifies a cryptographic publisher identity and detects post-sign tampering.
- **Apple notarisation** is an additional Apple automated malware/signature check; it is not a guarantee that app is harmless.
- **Open source** allows inspection but does not mean every shipped binary has been independently audited or reproducibly built.

## No-charge baseline from first preview

1. Release from reviewed immutable Git tag; lock dependencies; build on standard public-repo GitHub-hosted runners.
2. For each binary create checksum and signed GitHub provenance attestation with permissions `attestations: write`, `id-token: write`, `contents: read` on build job. Keep publish job's `contents: write` scope separate.
3. Release includes `SHA256SUMS.txt`, exact git commit, toolchain/runner info, signed/unsigned status per OS, supported OS and smoke-test matrix.
4. Ensure file names in checksum list exactly match published assets; attested file bytes must be the same ones users download. Verify each attestation **after publishing** using official GitHub CLI.
5. No release-from-PR secret exposure. Dependabot PRs must not have signing privileges.

## Windows signing routes

- **Unfunded first preview:** unsigned NSIS EXE, may encounter SmartScreen/Smart App Control warnings; do not advertise as no-warning installer.
- **Free possibility:** apply to [SignPath Foundation](https://signpath.org/) for OSS code signing after the project has actual maintained public releases and meets eligibility requirements. SignPath approval is **not automatic**, may have approvals/governance obligations, and foundation is signer/publisher.
- **Commercial alternative:** purchase own code signing solution later if necessary; not part of core £0 runtime stack.

## macOS signing/notarisation

- Conventional consumer-friendly distribution uses an Apple-issued **Developer ID Application** certificate and Apple's notarisation service. Apple Developer Program is typically US$99/year (local tax/currency may change). A plain unsigned/ad-hoc DMG is not equivalent.
- Sign and notarise on protected release workflow/macOS runner only after ownership, entitlements and credentials established. Use Apple's modern `notarytool` mechanism where configured through Tauri tooling. Test Gatekeeper on clean Mac.
- **Never** put Apple credentials, p12 passwords or certificates in source control; restrict secrets to protected GitHub environment and reviewed release tags.

## Verification commands for end users

```bash
# On macOS, after downloading a real release file:
shasum -a 256 Pancake-Compressor_v1.0.0_macos_arm64.dmg
# GitHub CLI provenance check; replace OWNER and version:
gh attestation verify Pancake-Compressor_v1.0.0_macos_arm64.dmg --repo OWNER/pancake-compressor
# Only signed app after installing:
codesign --verify --deep --strict --verbose=2 '/Applications/Pancake Compressor.app'
spctl --assess --type execute --verbose '/Applications/Pancake Compressor.app'
```

```powershell
# After downloading a real Windows EXE:
Get-FileHash '.\Pancake-Compressor_v1.0.0_windows_x64-setup.exe' -Algorithm SHA256
Get-AuthenticodeSignature '.\Pancake-Compressor_v1.0.0_windows_x64-setup.exe'
# The signature output reports NotSigned for unsigned previews.
```

## Release page trust table (required)

| Version | Windows signer | Mac ARM signing/notarisation | Mac Intel signing/notarisation | Attestations | Manual install checks |
|---|---|---|---|---|---|
| v0.x preview | record truth | record truth | record truth | link verification | device/OS/report |

Do not populate with green checkmarks until proven.

**Unsigned Mac technical preview note:** Tauri supports ad-hoc signing via `APPLE_SIGNING_IDENTITY=-` (also possible via `bundle.macOS.signingIdentity: "-"`). It is NOT notarisation and users may still be blocked by Gatekeeper. Source: https://v2.tauri.app/distribute/sign/macos/

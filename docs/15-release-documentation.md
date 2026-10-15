# 15 — Documentation users must see

**Publish these as GitHub Wiki/docs pages or at `docs/user/` when the actual application exists.** Do not invent screenshots/working instructions without installed-build checks.

## Download page template

# Pancake Compressor [version]

Pancake Compressor is a free, local-only image optimiser. Processing runs on your computer; no accounts or uploads required. [Link to licence, source, privacy, security, release notes]

Choose your device:

- **Windows 11 (64-bit):** `Pancake-Compressor_vX.Y.Z_windows_x64-setup.exe` — NSIS installer. Windows 10 not officially supported unless separately tested.
- **Mac with Apple Silicon (M1/M2/M3/M4…):** `Pancake-Compressor_vX.Y.Z_macos_arm64.dmg`.
- **Mac with Intel processor:** `Pancake-Compressor_vX.Y.Z_macos_x64.dmg`.

### Important security status

**Windows:** [signed by [identity] / unsigned developer preview — Windows may warn].
**macOS:** [signed and notarised / unsigned preview — Gatekeeper may block].
**Build integrity:** SHA256SUMS.txt and `gh attestation verify` instructions.

Only download from this project's canonical GitHub Releases page. No bundles from third-party mirrors. Do not advise disabling system security as a general installation step.

## Windows install steps

1. Download matching x64 `.exe`; verify source/hash if desired.
2. Run installer; follow prompts; launch Pancake Compressor.
3. Drop files/folders or choose via buttons; select format/preset; compress; open `_pancake` output.
4. To uninstall, use Windows Settings → Apps → Installed apps. Your exported images remain.

## macOS install steps

1. Apple menu → About This Mac → identify Apple Silicon vs Intel.
2. Download correct `.dmg`, inspect signature status and checksums.
3. Open disk image; drag Pancake Compressor.app to Applications; eject image.
4. Open from Applications. Signed/notarised release should behave according to normal Gatekeeper checks. Unsigned previews may require extra platform-specific user approval; direct nontechnical users to signed release instead of weakening Gatekeeper.
5. To uninstall, quit app and remove from Applications; settings cache may remain locally.

## Help topics to ship

- Supported formats / animated image limitations / lossy vs lossless.
- How Smart Auto and format selection work; sizes can sometimes increase.
- Where outputs go and how to change destination; no original overwritten by default.
- Why AVIF may be slower; handling memory limits, disk full and locked folders.
- Metadata/colour-profile implications and privacy mode.
- Offline/privacy promise and how to verify download authenticity.
- Accessibility, keyboard shortcuts and reduced-motion preference.
- Troubleshooting Windows WebView2, Mac Intel-vs-ARM selection, corrupt files, signing warnings.

## Release notes template

Include source tag + commit, changes, fixed issues/security advisories, known limitations, supported OSes, SHA256 of each asset, attestation verification command, signing/notarisation status, offline smoke test result and tested device versions.

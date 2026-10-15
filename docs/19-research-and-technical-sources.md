# 19 — Primary sources and verification log

These links are documentation inputs as checked for the **13 October 2026** planning handoff. The coding agent must verify exact API and dependency versions again when coding because build tools and hosted runners change.

## Framework and distribution

- Tauri v2 GitHub Actions packaging: https://v2.tauri.app/distribute/pipelines/github/
- Official tauri-action examples (currently `tauri-apps/tauri-action@v1`): https://github.com/tauri-apps/tauri-action
- Tauri Windows NSIS and WebView2 offline installer: https://v2.tauri.app/distribute/windows-installer/
- Tauri config schema: https://v2.tauri.app/reference/config/
- Tauri Content Security Policy: https://v2.tauri.app/security/csp/
- Tauri macOS signing: https://v2.tauri.app/distribute/sign/macos/
- Tauri Windows signing: https://v2.tauri.app/distribute/sign/windows/
- GitHub hosted runner names / public repo standard runner quota: https://docs.github.com/en/actions/reference/runners/github-hosted-runners

## Provenance and security

- GitHub artifact attestations, current `actions/attest@v4`: https://github.com/actions/attest
- GitHub artifact attestation guide: https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations
- SignPath Foundation code signing terms: https://signpath.org/terms.html
- Dependabot config: https://docs.github.com/en/code-security/dependabot/working-with-dependabot/dependabot-options-reference
- RustSec advisory database: https://rustsec.org/
- cargo-deny project: https://github.com/EmbarkStudios/cargo-deny
- OWASP File Upload Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html

## Image processing

- libimagequant licence: https://github.com/ImageOptim/libimagequant
- imagequant Rust bindings docs: https://docs.rs/imagequant/latest/imagequant/
- oxipng: https://github.com/oxipng/oxipng
- mozjpeg: https://github.com/mozilla/mozjpeg
- WebP reference and libwebp: https://developers.google.com/speed/webp
- ravif crate: https://docs.rs/ravif/latest/ravif/
- Rust image crate format feature support: https://docs.rs/image/latest/image/
- Exif and ICC: confirm support against each chosen encoder; conversion does not imply equivalent metadata preservation.

## Key verified caveats

1. SignPath Foundation offers *conditional* free open-source signing; a public released and actively maintained project and governance are among its requirements. Approval must not be assumed.
2. Normal macOS consumer distribution uses Developer ID signing/notarisation with paid Apple programme membership; unsigned/ad-hoc signed builds remain unsuitable for promise of a warning-free experience.
3. Windows 2026 SmartScreen may still warn even if a build is signed; reputation matters.
4. macOS runner `macos-15-intel` and Apple Silicon runner `macos-15` were listed as standard public-repo hosted runners. Recheck availability.
5. `imagequant` has GPLv3-or-later open-source licensing; do not default to MIT without a full change to codec/legal strategy.
6. Windows default Tauri WebView2 downloader can require an internet connection to install; `offlineInstaller` increases installer size significantly but enables an offline install.
7. No compression codec is a guarantee that every image becomes smaller; application must report unchanged/skipped outputs honestly.

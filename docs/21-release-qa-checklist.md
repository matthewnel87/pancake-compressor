# 21 — Human release readiness checklist (copy per release)

Release version: ____  Commit SHA: ____  Test owner: ____  Date: ____

## Integrity / build

- [ ] Source tag protected and build workflow green for all three architectures
- [ ] Windows x64 NSIS EXE present with correct icon/name/version
- [ ] macOS ARM64 DMG present, app installs/opens correctly on ARM machine
- [ ] macOS Intel DMG present, app installs/opens correctly on Intel machine
- [ ] `SHA256SUMS.txt` matches the exact published binaries
- [ ] Each app file has an independently verifiable GitHub attestation
- [ ] Code-signing and notarisation status accurately labelled on release page
- [ ] No credentials, paths, personal images, or secrets in release logs/assets

## Functionality: repeat on EACH platform

- [ ] Fresh install and app launch **without network**, verify startup functionality
- [ ] PNG->PNG lossy/lossless, JPEG->JPEG, WebP->WebP, AVIF->AVIF
- [ ] Conversion to PNG/JPEG/WebP/AVIF from supported source formats
- [ ] Animation, unsupported metadata and AVIF errors handled honestly
- [ ] Batch 100 mixed files and nested folder with same named files in different paths
- [ ] Drop files and pick folder with keyboard/native dialog
- [ ] Correct measured progress and savings; reduced motion and screen reader alternatives
- [ ] Quality and web presets, no larger file default, never overwrite original
- [ ] Transparency, EXIF rotated photo, ICC, corruption/zero-byte rejection
- [ ] Cancel mid-batch; restart; no orphan temp files or source corruption
- [ ] Process limited RAM/high dimensions; graceful failure not crash
- [ ] Export to non-ASCII and long valid paths; case collisions and permissions errors
- [ ] App operates in disconnected environment with no outbound connections
- [ ] Uninstall/reinstall doesn't delete exported images

## Publishing decision

- [ ] Security/libraries and licence report reviewed
- [ ] User download instructions tested on each platform
- [ ] Known limitations listed; draft/prerelease used for unsigned builds
- [ ] All remaining P0/P1 issues closed or explicitly release-blocked
- [ ] Human approval recorded before publishing a draft

Result: **PASS / BLOCKED**. If blocked, list affected platform, reproduction and owner.

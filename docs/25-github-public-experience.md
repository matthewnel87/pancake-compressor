# 25 — GitHub for downloaders, reviewers and contributors

## Reader priority order
1. **Download**: clearly labelled Windows x64 / Mac ARM64 / Mac Intel; show exact version, status (preview/stable), signing/notarisation reality and OS requirements.
2. **Understand**: short one-sentence promise, original logo, real app screenshot, supported PNG/JPEG/WebP/AVIF, offline privacy statement and limitations.
3. **Trust**: link `SECURITY.md`, `PRIVACY.md`, `docs/SECURITY-OVERVIEW.md`, installation and checksum/attestation instructions, known issues and actual test evidence.
4. **Give feedback**: GitHub Discussions categories `Feedback`, `Questions`, `Show your results`, `Ideas`. File reproducible bugs using issue forms; offer guidance to remove private image content from attachments.
5. **Improve it**: `CONTRIBUTING.md`, architecture tour, code-commenting standard, development prerequisites, `good first issue`, licence and third-party notices.

## Repository front page (README)
- Hero: logo + product name + plain tagline. Mark **planned/no installers yet** until release exists.
- Download table above long engineering details; links must lead to actual published assets, not draft/placeholder downloads. GitHub supports `<repo>/releases/latest`, but don't show a dead link before first release.
- One authentic screenshot or GIF of the real software, with reduced-motion static alternative. No fake screenshots portrayed as shipped UI.
- 4–6 sentences explaining workflow and privacy, with links to FAQ and install guides.
- 'Safe to install?' goes to transparent trust notes; never promise antivirus approval or unverified safety.
- 'Feedback' directs general opinions to Discussions; bug reports to Issues; vulnerability details to **private** security report.
- Developer docs lower down under 'How it's built'.

## Discussion setup (manual GitHub step)
Enable Discussions in repository settings, then create categories `Feedback & Reviews`, `Help & Questions`, `Feature Ideas`, `Showcase`. Pin a welcome post explaining: no logs with full local paths, never share confidential photos, one issue per reproducible bug, and constructive feedback welcome. GitHub does **not** offer a native app-store rating mechanism by default; don't advertise star ratings as product reviews. Repository stars are bookmarks/endorsement, not verified ratings.

## Public issue/PR workflow
- Keep issue templates for bugs, features and user feedback (`.github/ISSUE_TEMPLATE/`).
- Add `good first issue`, `help wanted`, `bug`, `accessibility`, `security`, `windows`, `macos`, `design`, `documentation` labels; labels configured in actual repo by maintainer/agent with permission.
- Each PR: motivation, screenshot when appropriate, threat impact, OS test matrix, changes to docs, release notes relevance.
- Protect default branch; require CI and review; use least-privilege GitHub Actions permissions.
- Publish release notes with SHA256, attestations, third-party notices, tested OS versions, signing status and known issues.

## Before any public announcement
- Publish only from exact tagged commit and verify downloadable assets.
- Independently test install/open/compress/cancel/uninstall on Windows x64, macOS Intel, macOS ARM64.
- If macOS has no Developer ID signing/notarisation, do not claim seamless installation. Consider keeping builds as technical previews until signing is available.
- Do not use a live Download button for unbuilt apps. Keep source/docs openly browsable throughout development.

## No marketing server required
Start with GitHub README, Releases, Issues and Discussions — all available for a public repo. A separate GitHub Pages landing page is optional **later**, generated from the same trusted release manifest without analytics or user-content uploads.

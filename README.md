# Pancake Compressor

**Squash file sizes, not image quality.**

Free, open-source, offline desktop image compression and conversion for Windows and macOS. Designed for people preparing images for the web: drag, optimise, export. No subscriptions, accounts, trackers, paid tiers or software-imposed batch quotas.

> **CURRENT STATUS — BUILD SPECIFICATIONS, NOT AN INSTALLABLE APPLICATION.** This repository starter contains comprehensive product/engineering/brand guidance and release-workflow templates. The desktop app and real Windows/macOS installers **have not yet been built or verified**. The download table below becomes active only after validated GitHub Releases exist.

## Downloads — coming after verified builds

| Platform | Planned official asset | Status |
|---|---|---|
| Windows 11 x64 | NSIS `.exe` offline installer | **Not available yet** |
| macOS Apple Silicon | ARM64 `.dmg` | **Not available yet** |
| macOS Intel | x86_64 `.dmg` | **Not available yet** |

Once releases exist, the maintainer will replace this section with **verified GitHub Release asset links**, release SHA-256 and attestation guidance, and actual code-signing/notarisation status. **Downloading GitHub's source archive is not the same as installing the app.**

## What we're building
- **Smart Auto** compression without codec expertise.
- Still **PNG, JPEG, WebP and AVIF** optimisation and conversion, subject to tested format support.
- **Unlimited batches without paywalls** — subject to device memory/storage safety limits.
- Native drag-and-drop and folder import; keep nesting and protect original images.
- Clear file-by-file results, real batch progress, useful error messages and an original pancake animation that respects reduced-motion settings.
- Warm, stylish, light/dark desktop UI that remains quick during CPU-heavy work.
- Local-only processing and no account or image-upload service.

## See what it will look like
Design and animation instructions are in [`docs/22-brand-design-system-v3.md`](docs/22-brand-design-system-v3.md), [`docs/23-ui-ux-flows-performance-v3.md`](docs/23-ui-ux-flows-performance-v3.md) and [`docs/27-logo-and-brand-asset-production.md`](docs/27-logo-and-brand-asset-production.md). Real screenshots will be posted **after** native UI has been implemented and reviewed; no concept image is represented as a screenshot of shipped software.

## Privacy, trust and safe installation
Pancake Compressor is designed to process all image data locally, never deleting originals by default. The source and CI will be auditable, and each release will disclose real test and signing evidence.

Read: [`Security overview`](docs/SECURITY-OVERVIEW.md) · [`Security policy`](SECURITY.md) · [`Privacy`](PRIVACY.md) · [`Windows installation`](docs/INSTALL-WINDOWS.md) · [`Mac installation`](docs/INSTALL-MACOS.md) · [`Implementation status`](docs/IMPLEMENTATION-STATUS.md).

## Review it, ask questions, suggest improvements
We welcome people who simply want a faster workflow, not only developers. Read [`Reviews and feedback`](docs/REVIEW-AND-FEEDBACK.md). After repository setup, maintainers should **enable GitHub Discussions** for general comments and reviews; use [GitHub Issues](.github/ISSUE_TEMPLATE/) for reproducible bugs or concrete requests, and use GitHub's **private vulnerability reporting** for security concerns. Do not share private images or filesystem paths.

## Developers: build and improve it
The codebase will be documented with source comments explaining non-obvious design choices, codec behaviour, resource limits and security assumptions, rather than commenting every trivial line. Start with [`Architecture for contributors`](docs/ARCHITECTURE-FOR-CONTRIBUTORS.md), [`CONTRIBUTING.md`](CONTRIBUTING.md), [`Code commenting`](docs/24-code-commenting-and-maintainer-guide.md), [`Milestones`](docs/17-codex-execution-plan.md) and [`Coding-agent instructions`](AGENTS.md).

Both **Codex** (`AGENTS.md`) and **Claude Code** (`CLAUDE.md`) can use the same implementation specifications. The agent should first complete M0, build and test the shell, and never misrepresent untested components as working. See [`START-HERE.md`](START-HERE.md).

**Licence:** GPL-3.0-or-later is the intended licensing model because of proposed `imagequant` use; verify the complete dependency licence tree before distribution. See [`LICENSE`](LICENSE) and dependency guidance.

# Security policy

Pancake Compressor opens images that may come from anywhere, so security is a core feature, not an add-on. Plain-language summary: [docs/SECURITY-OVERVIEW.md](docs/SECURITY-OVERVIEW.md).

## Supported versions

The project is in **technical preview**. Only the latest release receives security fixes. When stable releases begin, this table will list which versions are supported.

| Version | Supported |
|---|---|
| Latest preview | Yes |
| Older previews | No: please update |

## Reporting a vulnerability

**Please report privately**, using GitHub's **Security → Advisories → Report a vulnerability** page for this repository. Do not open a public issue, and do not post exploit files, private images or personal paths publicly.

This is a volunteer, open-source project, so we cannot promise fixed response times. We will acknowledge reports as soon as we can, work with you on a fix, credit you if you wish, and publish an advisory for confirmed issues.

Especially welcome: crafted image files that crash the app, use excessive memory or time, or escape the selected or output folders; ways the web interface could reach files or commands it should not; problems in the release or build pipeline.

## How the app is protected

- Images are treated as hostile: content-based format detection, size and memory limits before decoding, refusal of animated and unsupported formats, and tests against damaged and malicious files ([evidence](docs/IMPLEMENTATION-STATUS.md)).
- Files are never overwritten, outputs are written atomically, and folders are walked without following links.
- The interface has no file-system, shell or network permission; file access happens only in Rust code, behind tokens.
- No network code, analytics or updater. The engine fails to compile if network access is enabled.
- Locked dependency versions, `cargo deny` and `npm audit` on every pull request, and Dependabot updates.
- Threat model: [docs/10-threat-model.md](docs/10-threat-model.md).

## Verifying downloads

Releases are built by GitHub Actions from this repository, with SHA-256 checksums and GitHub build attestations. Step-by-step instructions: [docs/VERIFY-DOWNLOAD.md](docs/VERIFY-DOWNLOAD.md). Download only from this repository's Releases page, not from mirrors.

These checks prove what was built and from which source. They do not prove the absence of bugs.

## Signing

Current previews are **not** signed with an Apple Developer ID or a Windows code-signing certificate, and are labelled as such. macOS notarisation needs a paid Apple Developer Program membership; Windows signing may be available free to open-source projects through the SignPath Foundation once the project is public. We will never describe an unsigned build as signed. See [docs/14-signing-and-provenance.md](docs/14-signing-and-provenance.md).

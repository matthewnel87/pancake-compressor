# 16 — Public GitHub repository, governance and automation

This is a *community software project*, not a SaaS product. Public source, public issue tracker, release notes and reproducible build instructions are part of the product.

## Create and protect the repository (M0)

1. Create a **public** repository named `pancake-compressor`; push the complete handoff and the scaffold. If a repo already exists, use it instead of creating a duplicate. Never force-push to overwrite user work.
2. Enable Discussions and GitHub Issues. Add topics `rust`, `tauri`, `image-compression`, `webp`, `avif`, `open-source`.
3. Protect `main`: required CI, reviews from maintainers, prevent force pushes, prevent tag rewriting, no release from unreviewed PRs.
4. Enable Dependabot security updates and secret scanning features available to the repository. Enable private vulnerability reporting where supported; do not publish 0-day details before patch.
5. Add `.github/CODEOWNERS` when actual maintainers/GitHub usernames are known; do not invent identities.
6. Use an individual or organisation-owned verified domain before finalising reverse-domain package identifier and certificates. Choose a provisional valid identifier for developer builds, document later change risks.
7. Keep all decisions and tests in public docs. Disclose caveats rather than suggesting the app has been professionally audited.

## Automated checks

- `ci.yml`: JS/TS build, lint and tests; Rust format, Clippy, unit/integration tests on real Windows/macOS runners; audit lockfiles; no outbound network runtime test when implemented.
- `release.yml`: invoked by reviewed immutable `vX.Y.Z` tag, build Windows NSIS exe / macOS arm64 DMG / macOS Intel DMG separately; fail if any artifact missing; attest actual output bytes; stage, checksum, attach to a **draft prerelease**.
- `dependabot.yml`: npm, Cargo and GitHub Actions ecosystem updates weekly; PR-only, no automated major merges.
- Release publication is a **manual approval gate**: two OS families (three arch targets) installer tests and security evidence must be recorded before changing draft to published. Branch protection does not guarantee signing; use protected environments when credentials are added.

## Trust and community

- `SECURITY.md` contains vulnerability reporting steps; configure GitHub private vulnerability reporting or a maintainer-controlled security address before accepting reports. Don't fabricate an email contact.
- All issue templates collect reproduction steps, OS/CPU, input format and **non-sensitive synthetic fixtures**; never request personal images.
- `CONTRIBUTING.md` documents local environment, build commands, PR expectations, test requirements and the licensing requirement for incoming contributions.
- Maintain labelled issue backlog: `M0-bootstrap`, `M1-codecs`, `M2-batches`, `M3-ui`, `M4-security`, `M5-release`, `bug`, `good-first-issue`, `security`, `accessibility`, `benchmark`.

## Security of GitHub Actions

- Grant least privilege at job level; no `pull_request_target` workflows that execute contributor code with elevated permissions.
- Action versions `@vN` are a **starting template**; for hardened final release resolve and pin each third-party action to a reviewed immutable SHA and document automated update policy.
- Build provenance attestation uses OIDC ephemeral identity; never commit signing secrets into the repository. Release job only needs `contents:write` to create a *draft* release. Build job only needs `contents:read` plus attestation permissions.
- Do not depend on GitHub CI as proof an image decoder is safe; use corpus fuzzing and runtime testing.

## Evidence repository conventions

Each release tag has `docs/evidence/<version>/` in a release branch or tagged evidence package containing:
- `matrix.md`: OS/CPU, VM/hardware, steps, results, testers
- `benchmark.md`: dataset provenance, tool versions, size/quality/time/memory measurements
- `security.md`: dependency audit, static tests, fuzz run duration, incident status
- `install.md`: installer SHA, signing status, install/uninstall results, Gatekeeper/SmartScreen notes

Avoid committing actual user images, privileged crash logs or private file paths.

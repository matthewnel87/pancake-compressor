# Changelog

All notable user-visible changes will be recorded here. Use semantic versions and include fixed security advisories when relevant.

## [Unreleased]

### M0 developer shell (2026-10-15, not released)
- Scaffolded the Tauri v2 + React/TypeScript/Vite desktop shell with a Rust workspace and a separate `pancake-core` engine crate.
- Added an accessible launch screen that states compression is not available yet, plus a read-only `get_app_info` command reporting version and offline/no-tracking status.
- Locked down the webview: strict Content Security Policy, no shell/fs/http/opener/updater plugins, minimal window capability.
- Pinned Node 24 and Rust 1.99.1; CI now uses those pins. Installers are staged in `release-artifacts/` (ADR-019).
- **Still no application release or installer for end users.**

### Specification pack

- Created the v2 Codex implementation specification for Pancake Compressor, including platform-specific installer workflows, security/threat model and release gates.
- **No application release or installer has yet been built in this handoff.**

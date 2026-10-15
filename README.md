# 🥞 Pancake Compressor

**Squash file sizes, not image quality.**

A free, open-source desktop app for Windows and macOS that compresses and converts images for fast websites. Drop in a folder, press one button, get smaller PNG, JPEG, WebP or AVIF files. Everything happens on your own computer: no uploads, no account, no tracking, no limits, no paid tier. Ever.

> **Status: early technical preview.** The app works end to end on our test machines, but it is **not yet code-signed**, has **not yet been install-tested on Windows by a person**, and some features are still being finished. Read [what is verified](docs/IMPLEMENTATION-STATUS.md) before relying on it. Feedback is very welcome.

## Download

[![Download for Windows](https://img.shields.io/badge/Download-Windows-0078D6?style=for-the-badge&logo=windows11&logoColor=white)](../../releases) [![Download for macOS](https://img.shields.io/badge/Download-macOS-000000?style=for-the-badge&logo=apple&logoColor=white)](../../releases)

Both buttons go to the same **[Releases](../../releases)** page. Pick the file for your computer:

| Your computer | Download |
|---|---|
| Windows 11 (64-bit) | `Pancake-Compressor_<version>_windows_x64-setup.exe` |
| Mac with Apple chip (M1, M2, M3, M4…) | `Pancake-Compressor_<version>_macos_arm64.dmg` |
| Mac with Intel chip | `Pancake-Compressor_<version>_macos_x64.dmg` |

Not sure which Mac you have? Apple menu → **About This Mac** → look for "Chip" (Apple) or "Processor" (Intel).

The green **Code** button downloads the source code, not the app.

Installation help: [Windows](docs/INSTALL-WINDOWS.md) · [macOS](docs/INSTALL-MACOS.md)

## Why you can trust it

We would rather earn trust with evidence than ask for it. In short:

- **Open source.** Every line is public under GPL-3.0-or-later. Anyone can read, audit or rebuild it.
- **Built by GitHub, not on someone's laptop.** Release files are built by GitHub Actions from the public source, and each one has a **SHA-256 checksum** and a **GitHub build attestation** you can check yourself. [How to verify your download](docs/VERIFY-DOWNLOAD.md).
- **Offline by design.** The app contains no network code, analytics or update checker. The engine has a compile-time check that fails the build if that ever changes.
- **Your originals are never touched.** Results go to a separate `_pancake` folder and never overwrite anything.
- **Hostile files are expected.** Every image is checked before it is opened, with limits on size and memory, and the code is tested against damaged and malicious files.
- **Honest about what is not done yet.** The app is not yet signed with an Apple or Microsoft certificate, so your computer will warn you the first time. We explain exactly what that means and never ask you to switch off your system's protections.

Full details: [Security overview](docs/SECURITY-OVERVIEW.md) · [Privacy](PRIVACY.md) · [Security policy](SECURITY.md) · [What is verified](docs/IMPLEMENTATION-STATUS.md)

## What it does

- **Smart Auto:** picks the smallest suitable format per image, with rule-based choices you can see. Transparent images never get turned into JPEG.
- **PNG, JPEG, WebP and AVIF** in and out, using leading open-source encoders: MozJPEG, oxipng, imagequant (pngquant), libwebp and rav1e.
- **Never larger:** if a file cannot be made smaller, the original is kept and you are told.
- **Whole folders at once,** keeping subfolders, with real progress and honest results.
- **Before/after compare,** so you can check quality yourself.
- **Keeps what matters:** colour profiles, camera details and copyright info are kept where the format allows, and you are told when something cannot be kept.

## Help make it the best

This project has one goal: **the best free, lightweight image compression and conversion app for fast websites ever made.** It is free and open source, and it always will be. That is our [promise](PROMISE.md).

You are invited, whoever you are:

- **Try it and tell us what you think:** [Discussions](../../discussions) for ideas and reviews, [Issues](../../issues) for bugs.
- **Improve it:** code, design, accessibility, testing, translations and documentation are all welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md).
- **Build on it:** fork it, remix it or take the idea further. The GPL licence means you can, as long as what you share stays free and open source too.

## For developers

Requirements: Node 24 LTS (see `.nvmrc`), Rust 1.99.1 (installed automatically by `rustup` from `rust-toolchain.toml`), the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS, and on **Windows and Intel Macs** [NASM](https://www.nasm.us/) 2.14 or later on your `PATH` (used to build the AV1 codecs; not needed on Apple Silicon).

```bash
npm ci                      # install front-end dependencies from the lockfile
npm run tauri dev           # run the desktop app with hot reload
npm run lint && npm run test && npm run build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo deny check            # licences, advisories, sources
```

Start with the [architecture tour](docs/ARCHITECTURE-FOR-CONTRIBUTORS.md). Coding agents follow [AGENTS.md](AGENTS.md).

## Licence

[GPL-3.0-or-later](LICENSE). Third-party components and their licences are listed in [NOTICES.md](NOTICES.md).

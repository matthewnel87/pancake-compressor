# 05 — Repository structure and M0 scaffold

## Target tree

```text
pancake-compressor/
├── README.md  AGENTS.md  LICENSE  SECURITY.md  PRIVACY.md
├── CONTRIBUTING.md  CHANGELOG.md  .gitignore
├── package.json  package-lock.json  tsconfig*.json  vite.config.ts
├── index.html  src/  public/  tests/ui/
├── src-tauri/
│   ├── Cargo.toml  tauri.conf.json  build.rs  icons/
│   ├── capabilities/main.json
│   └── src/{lib.rs,main.rs,commands.rs,batch.rs}
├── crates/pancake-core/
│   ├── Cargo.toml
│   ├── src/{lib.rs,scan.rs,format.rs,limits.rs,metadata.rs,output.rs}
│   ├── src/encode/{mod.rs,png.rs,jpeg.rs,webp.rs,avif.rs}
│   └── tests/
├── fixtures/{README.md,synthetic/}
├── docs/ (these specifications + implementation status)
├── .github/{workflows,ISSUE_TEMPLATE,dependabot.yml}
└── scripts/ (release/smoke fixture tooling)
```

## Steps for M0

1. Confirm versions from official docs: `node --version`, `npm --version`, `rustc --version`, `cargo --version`, `npm create tauri-app@latest`. Scaffold Tauri v2 React+TypeScript using `npm`; copy this packet's Markdown governance/docs into the scaffold.
2. Create workspace root Cargo.toml with `src-tauri` and `crates/pancake-core` members; ensure crate path connections correct. Keep `package-lock.json` + `Cargo.lock` checked in.
3. Pin Node LTS major via `.nvmrc` and Rust toolchain via `rust-toolchain.toml` after build proof. Avoid hard-coded old versions copied from an AI answer.
4. Configure bundle identifier using an **owned reverse-domain identifier** (`io.github.<maintainer>.pancakecompressor` is a placeholder; do not publish the placeholder). Set product name, version, native icons, and OS-specific bundles.
5. Configure no updater / HTTP / shell. Restrict Tauri capabilities to `main` window, core and minimal OS dialogue. Avoid wildcard path scopes.
6. Make the first Rust core unit test and simple UI rendering test pass. GitHub CI must execute checks on the exact checked-in files.
7. Test a development build, then package dummy/bare window on all 3 target build runners before any codec work. Capture actual sizes and known warnings.

## Developer commands after scaffolding

```bash
npm ci
npm run dev
npm run tauri dev
npm run build
npm run test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

These commands are *acceptance targets*, not claims they execute in this docs-only packet. Define missing scripts during M0. Use `npm run tauri build -- --bundles nsis` on Windows and `--bundles dmg` on macOS; confirm precise CLI syntax for installed version.

## Packaging architecture decision

Build **natively on each OS** with GitHub runners rather than fragile cross-compilation. `macos-15` for ARM64 and `macos-15-intel` for Intel (confirm availability in repo plan); `windows-latest` for Windows x64. Make install smoke-tests on real or virtual target machines separate from CI's packaging success.

# How to check your download is genuine

You do not have to do this to use the app, but you can. Two checks, from easiest to strongest.

## 1. Checksum (proves the file was not damaged or swapped)

Each release has a `SHA256SUMS.txt` file listing a fingerprint for every download. Compare yours:

**macOS (Terminal):**
```bash
shasum -a 256 ~/Downloads/Pancake-Compressor_*_macos_arm64.dmg
```

**Windows (PowerShell):**
```powershell
Get-FileHash $HOME\Downloads\Pancake-Compressor_*_windows_x64-setup.exe -Algorithm SHA256
```

The long code printed must match the line for that file in `SHA256SUMS.txt` exactly. If it does not, do not open the file, and tell us privately (see [SECURITY.md](../SECURITY.md)).

## 2. Build attestation (proves GitHub built it from this repository's source)

Each release file has a signed record, created by GitHub Actions, saying which repository, workflow and commit produced it. With the [GitHub CLI](https://cli.github.com/) installed:

```bash
gh attestation verify ~/Downloads/Pancake-Compressor_<version>_macos_arm64.dmg --repo matthewnel87/pancake-compressor
```

A successful result names this repository and the release workflow. You can then open that commit and read exactly the code that was built.

## What these checks do and do not prove

- They **do** prove the file is the one GitHub built from the public source at a specific commit.
- They **do not** prove the code is bug-free or free of security problems. That is what open source review, the [security overview](SECURITY-OVERVIEW.md) and the [evidence ledger](IMPLEMENTATION-STATUS.md) are for.
- They are **not** the same as Apple notarisation or Windows code signing, which are not set up yet. See the install guides for what that means.

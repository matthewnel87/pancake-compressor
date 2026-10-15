# Security and privacy overview

Plain-language summary of how Pancake Compressor protects you, what has been tested, and what is not done yet. Updated for each release. Technical evidence: [IMPLEMENTATION-STATUS.md](IMPLEMENTATION-STATUS.md).

## How it protects you

| Promise | How it is enforced | Evidence |
|---|---|---|
| **No network, no tracking** | No HTTP client, analytics, crash uploader or updater in the app. The engine has a compile-time check (`RUNTIME_NETWORK_ACCESS`) that breaks the build if network access is ever enabled. The window's Content Security Policy allows no remote content. | Source; build checks; Content Security Policy in `src-tauri/tauri.conf.json` |
| **Originals never changed** | Results go to a separate folder. Files are written to a temporary name, flushed, then moved into place with an operation that **refuses to overwrite** anything. | Tests: originals byte-identical after a batch; 50 simultaneous writers never overwrite each other |
| **No partial or broken outputs** | Every output is decoded again and checked before it is kept. Lossless modes must match the original pixel for pixel. | Encoder tests and an all-formats matrix test on Windows, macOS ARM and macOS Intel |
| **Damaged or malicious images handled safely** | File type is decided by content, not name. Size and memory limits are checked before decoding. A tiny file claiming to be 100,000 × 100,000 pixels is refused. Animated images are refused, never silently cut down. | Tests on every truncation of every format, 20,000 random inputs, crafted "decompression bomb" files |
| **The web page cannot reach your files** | The interface runs in a locked-down web view. File pickers and drag-and-drop are handled in Rust; the page only gets an opaque token. No file-system, shell or network permission is granted to the page. | `src-tauri/capabilities/main.json`; bridge tests |
| **Folders are walked safely** | Shortcuts/symlinks are never followed, output folders are never re-read, and limits stop runaway folder trees. | Scanner tests incl. symlink loops and links pointing outside the folder |
| **One bad file cannot crash the app** | Each file is processed separately; an internal error in a codec becomes one failed file, not a crash. | Batch tests |
| **Honest metadata handling** | Colour profiles and camera/copyright info are kept where the format allows. If something cannot be kept, you are told. | Metadata round-trip tests |
| **Supply chain checked** | Exact dependency versions are locked. Every pull request runs `cargo deny` (known vulnerabilities, licences, sources) and `npm audit`. Dependabot proposes updates. | CI on every pull request |
| **Builds you can verify** | Releases are built by GitHub Actions from the public source, with SHA-256 checksums and GitHub build attestations. | [Verify your download](VERIFY-DOWNLOAD.md) |

## Not done yet (be aware)

- **Not code-signed.** macOS will say the app is from an unidentified developer and Windows SmartScreen may warn. Signing needs an Apple Developer account and a Windows signing service; we will say clearly when that changes.
- **Not yet install-tested on Windows or an Intel Mac by a person.** Automated builds and tests pass on both; a human install check is still to do.
- **Not yet fuzz-tested or independently audited.** Fuzzing of the file parsers is the next security milestone (M4).
- **No security claims beyond the above.** Open source and attestations show what was built; they do not prove the absence of bugs.

## Reporting a problem

- **Security issue:** report privately via the repository's **Security → Report a vulnerability** page. Do not post details publicly. See [SECURITY.md](../SECURITY.md).
- **Bug:** [Issues](../../../issues) (please do not attach private photos).
- **Ideas and reviews:** [Discussions](../../../discussions).

**Never turn off Gatekeeper or SmartScreen for your whole computer** to install this or any app. The install guides show the safe, per-app way.

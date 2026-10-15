# Install on Windows

> No installer is included in this build-plan pack. Use this guide **only after** a real published release and verified installation record exist.

1. Open the project's verified GitHub repository and choose **Releases**. Select a published version suitable for **Windows 11 x64** (Windows 10 is out of initial supported-security scope).
2. Download the Windows NSIS `.exe` from that release's **Assets**. Do **not** download source `.zip` expecting it to be the app.
3. Read the release's publisher/signature status. If the version is an unsigned technical preview, expect SmartScreen warnings; do not disable SmartScreen globally. Use it only if you understand and accept the risks, or wait for a signed version.
4. Optionally verify `SHA256SUMS` against the downloaded EXE using PowerShell: `Get-FileHash .\PancakeCompressor-<version>-windows-x64.exe -Algorithm SHA256` (replace example with **actual** asset name). Compare the result with that release's verified manifest.
5. Run the installer and launch the app. The first compression should work with network disconnected and write to a separate `_pancake` folder. Test with non-sensitive images.
6. Report crashes via the bug report issue form after removing private paths/images from logs.

Maintainers must replace generic file references with real artifact names, screenshots, verified OS builds and signature identity **before publishing stable releases**.

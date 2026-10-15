# Install on macOS

> No installer is included in this handoff. Release documents must specify actual signing/notarisation and tested OS support before publication.

1. Determine whether the Mac is **Apple Silicon** (M-series) or **Intel** using Apple menu → About This Mac.
2. From the official public GitHub **Releases** page, download the matching **ARM64** or **x86_64** `.dmg` asset, not the GitHub source archive.
3. Check release identity: version, hash, attestation, Developer ID signing and Apple notarisation status. Developer ID signing/notarisation requires a separate provisioning process; it is not automatically conferred by GitHub builds.
4. Open the DMG, drag the app to Applications if the installer instructs that, and open it. **Never disable Gatekeeper globally.** If Gatekeeper refuses to launch an unsigned preview, a notarised release is the recommended route for ordinary users.
5. Test an image with networking disconnected. Confirm the `_pancake` output folder and that the original remains unchanged.
6. Report issues via Issues with macOS version, Intel/ARM chip type and a reproducible non-private sample where possible.

Maintainer gate: clean-install and basic compression/uninstall tests on **both** Mac architectures before presenting the release as supported. The existence of a DMG does not establish it can launch on consumers' machines.

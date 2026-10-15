# Install on macOS

Works on macOS 13 (Ventura) and later.

1. **Check your chip:** Apple menu → **About This Mac**. "Chip: Apple M…" means Apple silicon; "Processor: Intel" means Intel.
2. **Download** from [Releases](../../../releases):
   - Apple silicon: `Pancake-Compressor_<version>_macos_arm64.dmg`
   - Intel: `Pancake-Compressor_<version>_macos_x64.dmg`
3. *(Optional)* [Verify the download](VERIFY-DOWNLOAD.md).
4. **Open the .dmg** and drag **Pancake Compressor** onto **Applications**. Eject the disk image.
5. **First launch.** The current previews are **not notarised by Apple**, so macOS will block the first open with a message about an unidentified developer. To allow just this app:
   - Open **System Settings → Privacy & Security**, scroll to the message about Pancake Compressor, and click **Open Anyway**. Then confirm.
   - This allows this one app only. **Never** turn off Gatekeeper for your whole Mac.
   - If you are not comfortable doing this, please wait for a notarised release.
6. **Try it:** drag a few images onto the window and press **Compress images**. Your results appear in a `_pancake` folder next to them; the originals are unchanged.

To remove the app, drag it from Applications to the Bin. Your compressed images are not deleted.

**Problems?** Tell us in [Issues](../../../issues) with your macOS version and chip type. Please do not attach private photos.

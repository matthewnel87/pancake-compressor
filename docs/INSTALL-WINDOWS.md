# Install on Windows

For 64-bit Windows 11 (Windows 10 may work but has not been tested). Installs for your user account only, so it does not need administrator rights.

1. **Download** `Pancake-Compressor_<version>_windows_x64-setup.exe` from [Releases](../../../releases). (The green **Code** button gives you source code, not the app.)
2. *(Optional)* [Verify the download](VERIFY-DOWNLOAD.md).
3. **Run the installer.** The current previews are **not code-signed**, so Microsoft Defender SmartScreen may show "Windows protected your PC". To continue for this file only: click **More info**, check the app name is *Pancake Compressor*, then **Run anyway**.
   - **Never** turn off SmartScreen for your whole computer.
   - If you are not comfortable doing this, please wait for a signed release.
4. The installer is large (about 220 MB) because it includes Microsoft's WebView2 component, so it can install without an internet connection.
5. **Try it:** drag a few images onto the window and press **Compress images**. Results go to a `_pancake` folder next to them; the originals are unchanged.

To uninstall, use **Settings → Apps → Installed apps → Pancake Compressor → Uninstall**. Your compressed images are not deleted.

**Problems?** Tell us in [Issues](../../../issues) with your Windows version. Please do not attach private photos.

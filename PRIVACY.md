# Privacy

**Short version: Pancake Compressor does not collect anything. Your images never leave your computer.**

## What the app does with your data

- **Images:** read from the files and folders you choose, processed in memory on your computer, and written to the output folder you choose (by default a `_pancake` folder next to them). Originals are never changed.
- **Previews:** small before/after previews are made in memory when you press Compare and are not saved to disk.
- **Settings:** your light/dark theme choice is saved in the app's local storage on your computer. Nothing else is saved between sessions.
- **Network:** none. The app has no network code, no analytics, no crash reporting, no update checker and no account. It works the same with your internet switched off.
- **Logs:** the app does not write logs containing your file names, paths or image content.

## Image metadata

Photos can contain hidden information such as the camera model, date, copyright and **GPS location**. By default the app **keeps** this information in the compressed copies where the format allows, because silently deleting it can lose things people care about (such as copyright). When it cannot be kept, the app tells you. An option to **remove metadata, including location**, before sharing is planned.

## Outside the app

Your operating system, antivirus and GitHub (when you download) have their own privacy practices. This statement covers the Pancake Compressor app itself.

## Changes

Any future feature that touches privacy (for example an optional update check) would be off by default, clearly explained, and recorded here and in the [decision log](docs/02-decisions-and-open-questions.md) first.

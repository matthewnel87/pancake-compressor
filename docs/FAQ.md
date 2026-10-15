# Frequently asked questions

**Is it free?** Yes, the project is intended to be fully free/open source and contain no premium features, accounts, subscriptions or usage quota. No application binaries are available from this handoff yet.

**Does it upload my photos?** The intended installed app processes them on your machine without uploading to a server. This must be verified and documented for each release.

**What formats?** Planned: still PNG, JPEG, WebP and AVIF input and output, subject to a publicly documented format matrix and codec tests.

**What does Smart Auto do?** It will test safe permitted compression candidates and choose results according to a documented quality/size policy. Its exact behaviour will be published with the app.

**Will it delete my originals?** No; the default is export to a separate `_pancake` directory. Any overwrite ability would require explicit opt-in, if added at all.

**Does unlimited mean I can process a 100 GB image?** There are no artificial software quotas; the actual limits depend on memory/CPU/storage and safe resource controls.

**Can I use it for commercial work?** Using GPL-licensed software to process commercial images does not make the images GPL-licensed. Redistribution or modification of the software has separate licence obligations; see `LICENSE` and `CONTRIBUTING.md`.

**How do I tell if an installer is genuine?** Only download published release assets from the official project repository, and consult release signatures/hashes/build attestations and test evidence. Do not assume every open-source binary is safe.

**Why did antivirus or Gatekeeper warn me?** Code signing/notarisation and Windows reputation affect warnings. See per-release statements and install guides. Avoid turning off OS-wide protections.

**How can I suggest changes?** See `docs/REVIEW-AND-FEEDBACK.md` for Discussions, Issues and Pull Requests.

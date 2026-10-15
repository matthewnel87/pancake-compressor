# Privacy — Pancake Compressor

**Local-only by design.** Pancake Compressor processes user-selected images on the same computer. The MVP has no app accounts, cloud processing, analytics, crash upload, advertising, tracking identifiers, or networking requirement. The app may keep local preference settings and short-lived image previews; document their paths and offer cleanup. The user selects export destinations and originals are never modified by default.

Scope note: operating systems and third-party download infrastructure may independently conduct security, update or diagnostic checks. This statement covers **the application** and not the user's entire computer. Verify the no-network assertion with source review, dependency audit, and an offline/firewall test on both platforms before each release. Do not claim a feature is implemented until it is verified.

## Image and metadata handling

Default behaviour and limitations must match `docs/07-image-format-contract.md`. Display when EXIF/ICC/XMP/location metadata will be kept or removed, and explain when conversion cannot preserve something. Any future crash diagnostics or update checks require a new privacy decision and clear user control.

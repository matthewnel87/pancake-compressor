/**
 * Plain-language wording for results, following the copy rules in
 * docs/12 and docs/22: say what happened and what the user can do, keep
 * jokes out of error messages, never blame the user.
 *
 * Codes come from Rust (`code()` methods in pancake-core). Unknown codes fall
 * back to a neutral message so a new Rust code never shows raw text.
 */
import type { FileState } from "./ipc";

const MESSAGES: Record<string, string> = {
  "unknown-format": "Not a PNG, JPEG, WebP or AVIF image, so it was skipped.",
  "unsupported-format": "This image format isn't supported yet. Your original was not changed.",
  "animated-image": "Animated images aren't supported yet. Your original was not changed.",
  "too-many-pixels": "This image is too large to process safely. Your original was not changed.",
  "memory-budget-exceeded": "This image needs more memory than the current limit allows.",
  "file-too-large": "The file is larger than 512 MB, so it was skipped.",
  "empty-file": "The file is empty.",
  "truncated-header": "This file looks damaged and couldn't be opened.",
  "malformed-header": "This file looks damaged and couldn't be opened.",
  "corrupt-image-data": "This file looks damaged and couldn't be opened.",
  "zero-dimensions": "This file looks damaged and couldn't be opened.",
  "dimension-mismatch": "This file contradicts itself about its size, so it wasn't opened.",
  "unsupported-image-feature": "This image uses a feature that can't be opened yet.",
  "decode-limit-exceeded": "This image needs more memory to open than is allowed.",
  "quality-not-reached": "A smaller version wasn't possible at this quality. Original kept.",
  "colour-profile-not-supported":
    "AVIF can't keep this image's colour profile yet. Choose another format, or allow colour changes.",
  "protected-content": "This image has signed content credentials, so it was left unchanged.",
  "output-exists": "A file with this name is already in the output folder, so it was skipped.",
  "output-permission-denied": "Can't write to this folder. Choose another output location.",
  "output-disk-full": "The disk is full. Free some space or choose another output location.",
  "output-read-only": "The output folder is read-only. Choose another output location.",
  "unsafe-output-path": "This file's folder can't be recreated safely in the output folder.",
  "output-write-failed": "The new file couldn't be saved.",
  "input-unreadable": "This file couldn't be read.",
  "original-already-smaller": "Already compact. Original kept.",
  "no-suitable-mode": "This preset can't be used for this format. Original kept.",
  "internal-error": "Something went wrong with this file. Your other images weren't affected.",
};

export function messageFor(code: string | null): string {
  if (code === null) return "";
  return MESSAGES[code] ?? "This file couldn't be processed.";
}

const NOTICES: Record<string, string> = {
  orientationApplied: "Rotated upright",
  transparencyFlattened: "Transparency replaced with white",
  precisionReduced: "Reduced to 8-bit colour",
  colourProfileDropped: "Colour profile not kept",
  metadataDropped: "Some metadata not kept",
};

export function noticeText(notice: string): string {
  return NOTICES[notice] ?? notice;
}

const STATE_LABELS: Record<FileState, string> = {
  decoding: "Opening",
  encoding: "Compressing",
  writing: "Saving",
  completed: "Done",
  notSmaller: "Kept original",
  skipped: "Skipped",
  failed: "Failed",
  cancelled: "Not started",
};

export function stateLabel(state: FileState | "queued"): string {
  return state === "queued" ? "Queued" : STATE_LABELS[state];
}

/** Human-readable size: 980 KB, 4.2 MB. Uses decimal units like Finder. */
export function formatBytes(bytes: number): string {
  if (bytes < 1000) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1000;
  let unit = 0;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit += 1;
  }
  const digits = value < 10 ? 1 : 0;
  return `${value.toFixed(digits)} ${units[unit]}`;
}

/** "Saved 18.4 MB (62%)" style summary of a saving; never negative. */
export function savingText(originalBytes: number, outputBytes: number): string {
  const saved = Math.max(0, originalBytes - outputBytes);
  const percent = originalBytes > 0 ? Math.round((saved / originalBytes) * 100) : 0;
  return `${formatBytes(saved)} (${percent}%)`;
}

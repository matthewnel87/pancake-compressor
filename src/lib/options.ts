/** Labels for the format and quality choices shown in the controls. */
import type { Preset, Target } from "./ipc";

export const TARGET_OPTIONS: { value: Target; label: string }[] = [
  { value: "auto", label: "Smart Auto (smallest suitable format)" },
  { value: "keep", label: "Keep original format" },
  { value: "webp", label: "WebP" },
  { value: "avif", label: "AVIF (smallest, slower)" },
  { value: "jpeg", label: "JPEG" },
  { value: "png", label: "PNG" },
];

export const PRESET_OPTIONS: { value: Preset; label: string }[] = [
  { value: "balanced", label: "Balanced" },
  { value: "smaller", label: "Smaller files" },
  { value: "bestQuality", label: "Best quality" },
  { value: "lossless", label: "Lossless (no visible change)" },
];

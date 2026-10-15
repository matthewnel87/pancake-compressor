/**
 * Typed wrappers for the Rust commands in `src-tauri/src/commands.rs`.
 *
 * The Rust side is the source of truth. Field names here must match the
 * camelCase serialisation of the Rust structs; when a Rust struct gains or
 * changes a field, bump its `schemaVersion` and update this file in the same
 * change.
 */
import { invoke } from "@tauri-apps/api/core";

/** Mirrors `AppInfo` in `src-tauri/src/commands.rs` (schema version 1). */
export interface AppInfo {
  schemaVersion: 1;
  appVersion: string;
  engineVersion: string;
  /** Always false: the installed app never uses the network. */
  networkAccess: boolean;
  /** Always false: no analytics or crash uploads. */
  telemetry: boolean;
}

/**
 * Asks the Rust shell for version and privacy facts.
 *
 * Rejects when the page is opened in a normal browser (`npm run dev`) rather
 * than inside the desktop app, because the Tauri IPC bridge does not exist
 * there. Callers should show a plain message instead of failing.
 */
export function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

/** Mirrors `InputSelection` in `src-tauri/src/batch.rs`. */
export interface InputSelection {
  /** Opaque: pass back to `startBatch`; never a path. */
  token: string;
  /** Name of the first chosen item, for display only. */
  label: string;
  fileCount: number;
  skippedCount: number;
  totalBytes: number;
  truncated: boolean;
}

/** Mirrors `OutputSelection` in `src-tauri/src/batch.rs`. */
export interface OutputSelection {
  token: string;
  label: string;
}

export type Target = "keep" | "png" | "jpeg" | "webp" | "avif" | "auto";
export type Preset = "balanced" | "smaller" | "bestQuality" | "lossless";

/** Mirrors `BatchOptionsDto`. Rust rejects unknown fields and values. */
export interface BatchOptions {
  inputToken: string;
  outputToken: string | null;
  target: Target;
  preset: Preset;
  conflict: "rename" | "skip";
  allowColourProfileLoss?: boolean;
  tryAvifInAuto?: boolean;
}

export type FileState =
  | "decoding"
  | "encoding"
  | "writing"
  | "completed"
  | "notSmaller"
  | "skipped"
  | "failed"
  | "cancelled";

/** Mirrors `BatchEventDto` (schema version 1), sent as "batch-event". */
export interface BatchEvent {
  schemaVersion: 1;
  batchId: string;
  /** Strictly increasing per batch; use it to drop late events. */
  sequence: number;
  fileIndex: number;
  /** Path relative to the chosen folder, for display. */
  file: string;
  state: FileState;
  code: string | null;
  originalBytes: number | null;
  outputBytes: number | null;
  outputFormat: string | null;
  notices: string[];
  /** Files with a final result so far. */
  processed: number;
  total: number;
}

/** Mirrors `SnapshotDto`, also sent as "batch-finished". */
export interface BatchSnapshot {
  batchId: string;
  total: number;
  processed: number;
  lastSequence: number;
  finished: boolean;
  completed: number;
  notSmaller: number;
  skipped: number;
  failed: number;
  cancelled: number;
  originalBytes: number;
  outputBytes: number;
}

/** Opens the native picker in Rust. Resolves to null if cancelled. */
export function selectInputs(kind: "files" | "folders"): Promise<InputSelection | null> {
  return invoke<InputSelection | null>("select_inputs", { kind });
}

export function selectOutputFolder(): Promise<OutputSelection | null> {
  return invoke<OutputSelection | null>("select_output_folder");
}

/** Starts a batch and resolves to its id; progress arrives as events. */
export function startBatch(options: BatchOptions): Promise<string> {
  return invoke<string>("start_batch", { options });
}

export function cancelBatch(batchId: string): Promise<void> {
  return invoke<void>("cancel_batch", { batchId });
}

export function getBatchSnapshot(batchId: string): Promise<BatchSnapshot> {
  return invoke<BatchSnapshot>("get_batch_snapshot", { batchId });
}

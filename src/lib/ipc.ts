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

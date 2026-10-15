/**
 * Pure reducer for batch progress shown in the UI.
 *
 * Rust sends events in strictly increasing `sequence` order, but the UI can
 * still see late events (from a cancelled earlier batch) or duplicates after
 * reconnecting. Events for another batch, or with a sequence not newer than
 * the last applied one, are ignored, so counts only move forward and always
 * add up to `total` when the batch ends.
 *
 * Progress is `processed / total`: finished files over all files. There is no
 * per-file percentage, because codecs report no reliable progress
 * (AGENTS.md rule 6).
 */
import type { BatchEvent, FileState } from "./ipc";

export interface FileRow {
  file: string;
  state: FileState;
  code: string | null;
  originalBytes: number | null;
  outputBytes: number | null;
  outputFormat: string | null;
  notices: string[];
}

export interface BatchProgress {
  batchId: string;
  lastSequence: number;
  processed: number;
  total: number;
  files: Record<number, FileRow>;
}

export function startProgress(batchId: string, total: number): BatchProgress {
  return { batchId, lastSequence: 0, processed: 0, total, files: {} };
}

const FINAL_STATES: ReadonlySet<FileState> = new Set([
  "completed",
  "notSmaller",
  "skipped",
  "failed",
  "cancelled",
]);

export function isFinal(state: FileState): boolean {
  return FINAL_STATES.has(state);
}

/** Applies one event; returns the same object when the event is ignored. */
export function applyEvent(progress: BatchProgress, event: BatchEvent): BatchProgress {
  if (event.batchId !== progress.batchId || event.sequence <= progress.lastSequence) {
    return progress;
  }
  const previous = progress.files[event.fileIndex];
  // A file that already has a final result keeps it.
  if (previous && isFinal(previous.state)) {
    return { ...progress, lastSequence: event.sequence };
  }
  return {
    ...progress,
    lastSequence: event.sequence,
    processed: event.processed,
    total: event.total,
    files: {
      ...progress.files,
      [event.fileIndex]: {
        file: event.file,
        state: event.state,
        code: event.code,
        originalBytes: event.originalBytes,
        outputBytes: event.outputBytes,
        outputFormat: event.outputFormat,
        notices: event.notices,
      },
    },
  };
}

/** Whole-number percentage of files finished, for the progress bar. */
export function percentDone(progress: BatchProgress): number {
  if (progress.total === 0) return 0;
  return Math.floor((progress.processed / progress.total) * 100);
}

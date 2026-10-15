import { describe, expect, it } from "vitest";
import { applyEvent, percentDone, startProgress } from "./batchProgress";
import type { BatchEvent } from "./ipc";

function event(partial: Partial<BatchEvent>): BatchEvent {
  return {
    schemaVersion: 1,
    batchId: "batch-1",
    sequence: 1,
    fileIndex: 0,
    file: "a.png",
    state: "decoding",
    code: null,
    originalBytes: null,
    outputBytes: null,
    outputFormat: null,
    notices: [],
    processed: 0,
    total: 2,
    ...partial,
  };
}

describe("batch progress reducer", () => {
  it("applies events in sequence and counts finished files", () => {
    let p = startProgress("batch-1", 2);
    p = applyEvent(p, event({ sequence: 1, state: "decoding" }));
    p = applyEvent(p, event({ sequence: 2, state: "completed", processed: 1, outputBytes: 10 }));
    p = applyEvent(p, event({ sequence: 3, fileIndex: 1, file: "b.png", state: "skipped", code: "unknown-format", processed: 2 }));
    expect(p.processed).toBe(2);
    expect(percentDone(p)).toBe(100);
    expect(p.files[0]?.state).toBe("completed");
    expect(p.files[1]?.code).toBe("unknown-format");
  });

  it("ignores late, duplicate and foreign events", () => {
    let p = startProgress("batch-1", 2);
    p = applyEvent(p, event({ sequence: 5, state: "encoding" }));
    const unchanged = p;
    expect(applyEvent(p, event({ sequence: 4, state: "decoding" }))).toBe(unchanged);
    expect(applyEvent(p, event({ sequence: 5, state: "writing" }))).toBe(unchanged);
    expect(applyEvent(p, event({ batchId: "batch-0", sequence: 99, state: "failed" }))).toBe(unchanged);
  });

  it("never lets a later event undo a final result", () => {
    let p = startProgress("batch-1", 1);
    p = applyEvent(p, event({ sequence: 1, state: "completed", processed: 1 }));
    p = applyEvent(p, event({ sequence: 2, state: "writing" }));
    expect(p.files[0]?.state).toBe("completed");
    expect(p.processed).toBe(1);
  });

  it("reports zero progress for an empty batch instead of dividing by zero", () => {
    expect(percentDone(startProgress("batch-1", 0))).toBe(0);
  });
});

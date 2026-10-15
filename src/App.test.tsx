import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import type { AppInfo, BatchEvent, BatchSnapshot, InputSelection } from "./lib/ipc";

// The Tauri bridge does not exist under jsdom, so the IPC module is mocked.
// `handlers` captures the event subscriptions so tests can play Rust's part.
const handlers = new Map<string, (payload: unknown) => void>();
const mocks = {
  getAppInfo: vi.fn<() => Promise<AppInfo>>(),
  selectInputs: vi.fn<(kind: string) => Promise<InputSelection | null>>(),
  selectOutputFolder: vi.fn(),
  startBatch: vi.fn<(options: unknown) => Promise<string>>(),
  cancelBatch: vi.fn<(id: string) => Promise<void>>(),
  openOutputFolder: vi.fn<(id: string) => Promise<void>>(),
};
vi.mock("./lib/ipc", () => ({
  getAppInfo: () => mocks.getAppInfo(),
  selectInputs: (kind: string) => mocks.selectInputs(kind),
  selectOutputFolder: () => mocks.selectOutputFolder(),
  startBatch: (options: unknown) => mocks.startBatch(options),
  cancelBatch: (id: string) => mocks.cancelBatch(id),
  openOutputFolder: (id: string) => mocks.openOutputFolder(id),
  getBatchSnapshot: vi.fn(),
  subscribe: (name: string, handler: (payload: unknown) => void) => {
    handlers.set(name, handler);
    return Promise.resolve(() => handlers.delete(name));
  },
}));

const selection: InputSelection = {
  token: "in-1",
  label: "Holiday",
  fileCount: 2,
  skippedCount: 0,
  totalBytes: 2_500_000,
  truncated: false,
};

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

const finished: BatchSnapshot = {
  batchId: "batch-1",
  total: 2,
  processed: 2,
  lastSequence: 4,
  finished: true,
  completed: 1,
  notSmaller: 0,
  skipped: 1,
  failed: 0,
  cancelled: 0,
  originalBytes: 2_000_000,
  outputBytes: 500_000,
};

async function emit(name: string, payload: unknown) {
  await act(async () => {
    handlers.get(name)?.(payload);
  });
}

describe("main window", () => {
  beforeEach(() => {
    handlers.clear();
    Object.values(mocks).forEach((mock) => mock.mockReset());
    mocks.getAppInfo.mockResolvedValue({
      schemaVersion: 1,
      appVersion: "0.1.0",
      engineVersion: "0.1.0",
      networkAccess: false,
      telemetry: false,
    });
  });

  it("starts with the product name and keyboard-reachable choose buttons", async () => {
    render(<App />);
    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent("Pancake Compressor");
    expect(screen.getByRole("button", { name: "Choose images" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Choose folder" })).toBeEnabled();
    expect(await screen.findByText(/Works offline · No tracking/)).toBeInTheDocument();
  });

  it("runs a batch from selection to an honest summary", async () => {
    const user = userEvent.setup();
    mocks.selectInputs.mockResolvedValue(selection);
    mocks.startBatch.mockResolvedValue("batch-1");
    render(<App />);

    await user.click(screen.getByRole("button", { name: "Choose images" }));
    expect(mocks.selectInputs).toHaveBeenCalledWith("files");
    expect(await screen.findByText(/2 files · 2\.5 MB from Holiday/)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Compress images" }));
    expect(mocks.startBatch).toHaveBeenCalledWith(
      expect.objectContaining({ inputToken: "in-1", target: "auto", preset: "balanced", outputToken: null }),
    );
    const bar = await screen.findByRole("progressbar", { name: "Images processed" });
    expect(bar).toHaveAttribute("aria-valuemax", "2");

    await emit("batch-event", event({ sequence: 1, state: "decoding" }));
    await emit("batch-event", event({ sequence: 2, state: "completed", processed: 1, originalBytes: 2_000_000, outputBytes: 500_000 }));
    await emit("batch-event", event({ sequence: 3, fileIndex: 1, file: "notes.txt", state: "skipped", code: "unknown-format", processed: 2 }));
    await emit("batch-finished", finished);

    expect(await screen.findByRole("heading", { name: /You saved 1\.5 MB \(75%\)/ })).toHaveFocus();
    const summary = within(screen.getByRole("region", { name: /You saved/ }));
    expect(summary.getByText("Compressed").nextSibling).toHaveTextContent("1");
    expect(summary.getByText("Skipped").nextSibling).toHaveTextContent("1");
    expect(screen.getByText("2.0 MB → 500 KB")).toBeInTheDocument();
    expect(screen.getByText(/Not a PNG, JPEG, WebP or AVIF image/)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Open output folder" }));
    expect(mocks.openOutputFolder).toHaveBeenCalledWith("batch-1");
  });

  it("ignores events from another batch", async () => {
    const user = userEvent.setup();
    mocks.selectInputs.mockResolvedValue(selection);
    mocks.startBatch.mockResolvedValue("batch-1");
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Choose images" }));
    await user.click(await screen.findByRole("button", { name: "Compress images" }));
    await emit("batch-event", event({ batchId: "batch-0", sequence: 9, state: "failed", processed: 2 }));
    await emit("batch-finished", { ...finished, completed: 0, skipped: 0, notSmaller: 2, originalBytes: 0, outputBytes: 0 });
    expect(screen.queryByText("Failed")).not.toBeInTheDocument();
  });

  it("explains a failed command in plain language", async () => {
    const user = userEvent.setup();
    mocks.selectInputs.mockResolvedValue(selection);
    mocks.startBatch.mockRejectedValue("unknown-input-token");
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Choose images" }));
    await user.click(await screen.findByRole("button", { name: "Compress images" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Please choose them again");
  });

  it("tells the user when a dropped folder had no files", async () => {
    render(<App />);
    await emit("inputs-dropped", { ...selection, fileCount: 0, totalBytes: 0 });
    expect(await screen.findByRole("alert")).toHaveTextContent("No files were found");
  });
});

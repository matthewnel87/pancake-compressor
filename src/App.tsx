/**
 * Pancake Compressor's single window (docs/23).
 *
 * Flow: add images (buttons, or drop onto the window) → choose format and
 * quality → Compress → watch real progress → summary with Open folder.
 *
 * All file access happens in Rust. This component holds only opaque tokens
 * and display labels, and turns Rust events into screen state through the
 * tested `applyEvent` reducer. Events are buffered and applied about ten
 * times a second so a large batch does not re-render on every event;
 * buffering never drops an event, so final results always arrive.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { Controls } from "./components/Controls";
import { DropZone } from "./components/DropZone";
import { PancakeMark } from "./components/PancakeMark";
import { ProgressPanel } from "./components/ProgressPanel";
import { Queue } from "./components/Queue";
import { Summary } from "./components/Summary";
import { applyEvent, startProgress, type BatchProgress } from "./lib/batchProgress";
import {
  cancelBatch,
  getAppInfo,
  openOutputFolder,
  selectInputs,
  selectOutputFolder,
  startBatch,
  subscribe,
  type AppInfo,
  type BatchEvent,
  type BatchSnapshot,
  type InputSelection,
  type OutputSelection,
  type Preset,
  type Target,
} from "./lib/ipc";
import { formatBytes } from "./lib/text";
import { THEME_LABELS, applyTheme, loadTheme, nextTheme, type ThemeChoice } from "./lib/theme";

type Stage = "start" | "ready" | "running" | "finished";

const FLUSH_MS = 100;

export default function App() {
  const [info, setInfo] = useState<AppInfo | null | "unavailable">(null);
  const [theme, setTheme] = useState<ThemeChoice>(() => loadTheme());
  const [stage, setStage] = useState<Stage>("start");
  const [selection, setSelection] = useState<InputSelection | null>(null);
  const [output, setOutput] = useState<OutputSelection | null>(null);
  const [target, setTarget] = useState<Target>("auto");
  const [preset, setPreset] = useState<Preset>("balanced");
  const [hovering, setHovering] = useState(false);
  const [progress, setProgress] = useState<BatchProgress | null>(null);
  const [snapshot, setSnapshot] = useState<BatchSnapshot | null>(null);
  const [cancelling, setCancelling] = useState(false);
  const [announcement, setAnnouncement] = useState("");
  const [error, setError] = useState<string | null>(null);

  const pending = useRef<BatchEvent[]>([]);
  const lastMilestone = useRef(-1);
  const summaryHeading = useRef<HTMLDivElement>(null);

  useEffect(() => applyTheme(theme), [theme]);

  useEffect(() => {
    let cancelled = false;
    getAppInfo()
      .then((value) => !cancelled && setInfo(value))
      .catch(() => !cancelled && setInfo("unavailable"));
    return () => {
      cancelled = true;
    };
  }, []);

  // Take buffered events out first, then apply them with a pure updater:
  // React may call updaters twice in development, so the updater itself
  // must not touch the buffer.
  const flushPending = useCallback(() => {
    if (pending.current.length === 0) return;
    const events = pending.current;
    pending.current = [];
    setProgress((current) => (current ? events.reduce(applyEvent, current) : current));
  }, []);

  const acceptSelection = useCallback((chosen: InputSelection | null) => {
    if (!chosen) return;
    setSelection(chosen);
    setStage(chosen.fileCount > 0 ? "ready" : "start");
    setError(chosen.fileCount > 0 ? null : "No files were found in what you chose.");
  }, []);

  // Rust → UI events. Subscriptions are set up once.
  useEffect(() => {
    const unsubscribers: Promise<() => void>[] = [
      subscribe<boolean>("drag-hover", setHovering),
      subscribe<InputSelection>("inputs-dropped", (chosen) => acceptSelection(chosen)),
      subscribe<BatchEvent>("batch-event", (event) => {
        pending.current.push(event);
      }),
      subscribe<BatchSnapshot>("batch-finished", (finished) => {
        flushPending();
        setSnapshot(finished);
        setStage("finished");
        setCancelling(false);
      }),
    ];
    return () => {
      unsubscribers.forEach((p) => p.then((stop) => stop()));
    };
  }, [acceptSelection, flushPending]);

  // Apply buffered events at a steady rate while a batch runs.
  useEffect(() => {
    if (stage !== "running") return;
    const timer = window.setInterval(flushPending, FLUSH_MS);
    return () => window.clearInterval(timer);
  }, [stage, flushPending]);

  // Announce progress at 10% steps for screen readers (docs/12).
  useEffect(() => {
    if (!progress || stage !== "running" || progress.total === 0) return;
    const step = Math.floor((progress.processed / progress.total) * 10);
    if (step !== lastMilestone.current) {
      lastMilestone.current = step;
      setAnnouncement(`Squashing your images… ${progress.processed} of ${progress.total} processed`);
    }
  }, [progress, stage]);

  useEffect(() => {
    if (stage === "finished") summaryHeading.current?.querySelector("h2")?.focus();
  }, [stage]);

  async function run<T>(action: () => Promise<T>): Promise<T | undefined> {
    try {
      setError(null);
      return await action();
    } catch (problem) {
      setError(typeof problem === "string" ? describeCommandError(problem) : "Something went wrong. Please try again.");
      return undefined;
    }
  }

  async function chooseInputs(kind: "files" | "folders") {
    const chosen = await run(() => selectInputs(kind));
    if (chosen !== undefined) acceptSelection(chosen);
  }

  async function chooseOutput() {
    const chosen = await run(() => selectOutputFolder());
    if (chosen) setOutput(chosen);
  }

  async function start() {
    if (!selection) return;
    const batchId = await run(() =>
      startBatch({
        inputToken: selection.token,
        outputToken: output?.token ?? null,
        target,
        preset,
        conflict: "rename",
      }),
    );
    if (!batchId) return;
    pending.current = [];
    lastMilestone.current = -1;
    setProgress(startProgress(batchId, selection.fileCount));
    setSnapshot(null);
    setStage("running");
  }

  async function cancel() {
    if (!progress) return;
    setCancelling(true);
    await run(() => cancelBatch(progress.batchId));
  }

  function startAgain() {
    setSelection(null);
    setProgress(null);
    setSnapshot(null);
    setAnnouncement("");
    setStage("start");
  }

  const busy = stage === "running";
  const outputLabel = output ? output.label : "_pancake folder next to your images";

  return (
    <div className="flex min-h-screen flex-col bg-surface text-ink">
      <header className="flex items-center gap-3 border-b border-line px-6 py-3">
        <PancakeMark className="h-8 w-8" />
        <h1 className="text-lg font-semibold">Pancake Compressor</h1>
        <p className="text-sm text-ink-muted">Local. Private. Free.</p>
        <button
          type="button"
          className="btn-secondary ml-auto"
          onClick={() => setTheme((t) => nextTheme(t))}
          aria-label={`${THEME_LABELS[theme]}. Change theme`}
        >
          {THEME_LABELS[theme]}
        </button>
      </header>

      <main className="mx-auto flex w-full max-w-4xl flex-1 flex-col gap-6 px-6 py-8">
        {error && (
          <p role="alert" className="rounded-card border border-error px-4 py-3 text-error">
            {error}
          </p>
        )}

        {(stage === "start" || stage === "ready") && (
          <DropZone
            hovering={hovering}
            busy={busy}
            onChooseFiles={() => chooseInputs("files")}
            onChooseFolder={() => chooseInputs("folders")}
          />
        )}

        {stage === "ready" && selection && (
          <>
            <p className="tabular-nums" role="status" aria-live="polite">
              {selection.fileCount} {selection.fileCount === 1 ? "file" : "files"} · {formatBytes(selection.totalBytes)}
              {selection.label && ` from ${selection.label}`}
              {selection.truncated && " (only the first 100,000 files were added)"}
            </p>
            <Controls
              target={target}
              preset={preset}
              outputLabel={outputLabel}
              disabled={busy}
              onTarget={setTarget}
              onPreset={setPreset}
              onChooseOutput={chooseOutput}
              onStart={start}
            />
          </>
        )}

        {stage === "running" && progress && (
          <ProgressPanel
            processed={progress.processed}
            total={progress.total}
            announcement={announcement}
            cancelling={cancelling}
            onCancel={cancel}
          />
        )}

        {stage === "finished" && snapshot && (
          <div ref={summaryHeading}>
            <Summary
              snapshot={snapshot}
              onOpenFolder={() => run(() => openOutputFolder(snapshot.batchId))}
              onStartAgain={startAgain}
            />
          </div>
        )}

        {progress && (stage === "running" || stage === "finished") && (
          <Queue total={progress.total} rows={progress.files} />
        )}
      </main>

      <footer className="border-t border-line px-6 py-3 text-sm text-ink-muted">
        <p className="tabular-nums">
          {info === null && "Reading version information…"}
          {info === "unavailable" && "Version information is only available in the desktop app."}
          {info !== null && info !== "unavailable" && (
            <>
              Version {info.appVersion} · Engine {info.engineVersion} ·{" "}
              {info.networkAccess ? "Uses the network" : "Works offline"} ·{" "}
              {info.telemetry ? "Sends usage data" : "No tracking"}
            </>
          )}
        </p>
      </footer>
    </div>
  );
}

/** Maps Rust command error codes to plain language. */
function describeCommandError(code: string): string {
  switch (code) {
    case "unknown-input-token":
      return "Those images are no longer available. Please choose them again.";
    case "output-folder-missing":
      return "The output folder can't be found. It may have been moved or deleted.";
    case "could-not-open-folder":
      return "The output folder couldn't be opened.";
    default:
      return "Something went wrong. Please try again.";
  }
}

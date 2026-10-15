/**
 * M0 launch screen.
 *
 * Shows the brand, an honest statement that compression is not built yet,
 * and version/privacy facts read from Rust. It deliberately offers no
 * "Choose images" button: AGENTS.md forbids presenting features that do not
 * work. The drop zone and queue replace the notice in M3.
 */
import { useEffect, useState } from "react";
import { PancakeMark } from "./components/PancakeMark";
import { getAppInfo, type AppInfo } from "./lib/ipc";

type InfoState =
  | { status: "loading" }
  | { status: "ready"; info: AppInfo }
  | { status: "unavailable" };

export default function App() {
  const [infoState, setInfoState] = useState<InfoState>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;
    getAppInfo()
      .then((info) => {
        if (!cancelled) setInfoState({ status: "ready", info });
      })
      .catch(() => {
        if (!cancelled) setInfoState({ status: "unavailable" });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="flex min-h-screen flex-col bg-surface text-ink">
      <main className="mx-auto flex w-full max-w-2xl flex-1 flex-col justify-center gap-8 px-6 py-10">
        <header className="flex items-center gap-4">
          <PancakeMark className="h-16 w-16 shrink-0" />
          <div>
            <h1 className="text-3xl font-semibold tracking-tight">Pancake Compressor</h1>
            <p className="mt-1 text-lg text-ink-muted">Squash file sizes, not image quality.</p>
          </div>
        </header>

        <section
          aria-labelledby="preview-notice-title"
          className="rounded-dropzone border-2 border-dashed border-line bg-surface-raised p-8"
        >
          <h2 id="preview-notice-title" className="text-xl font-semibold">
            Early developer build
          </h2>
          <p className="mt-2 text-ink-muted">
            Image compression is not available in this version yet. This build checks that the
            desktop app installs and opens correctly on Windows and macOS.
          </p>
        </section>

        <p className="text-sm font-medium text-ink-muted">Local. Private. Free.</p>
      </main>

      <footer className="border-t border-line px-6 py-3 text-sm text-ink-muted">
        <p role="status" aria-live="polite" className="tabular-nums">
          {infoState.status === "loading" && "Reading version information…"}
          {infoState.status === "unavailable" &&
            "Version information is only available in the desktop app."}
          {infoState.status === "ready" && (
            <>
              Version {infoState.info.appVersion} · Engine {infoState.info.engineVersion} ·{" "}
              {infoState.info.networkAccess ? "Uses the network" : "Works offline"} ·{" "}
              {infoState.info.telemetry ? "Sends usage data" : "No tracking"}
            </>
          )}
        </p>
      </footer>
    </div>
  );
}

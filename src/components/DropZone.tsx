/**
 * Where users add images: drop files or folders on the window, or use the
 * two buttons (the keyboard and screen-reader route; dragging is never
 * required, docs/12).
 *
 * Dropped paths are handled entirely in Rust (`handle_drag_drop`); this
 * component only shows the hover highlight Rust reports.
 */
import { PancakeMark } from "./PancakeMark";

interface DropZoneProps {
  hovering: boolean;
  busy: boolean;
  onChooseFiles: () => void;
  onChooseFolder: () => void;
}

export function DropZone({ hovering, busy, onChooseFiles, onChooseFolder }: DropZoneProps) {
  return (
    <section
      aria-labelledby="drop-title"
      className={`rounded-dropzone border-2 border-dashed p-8 text-center transition-colors ${
        hovering ? "border-accent bg-accent/10" : "border-line bg-surface-raised"
      }`}
    >
      <PancakeMark className="mx-auto h-14 w-14" />
      <h2 id="drop-title" className="mt-3 text-xl font-semibold">
        {hovering ? "Drop to add these images" : "Drop images or folders here"}
      </h2>
      <p className="mt-1 text-ink-muted">PNG, JPEG, WebP and AVIF. Your originals are never changed.</p>
      <div className="mt-5 flex flex-wrap justify-center gap-3">
        <button type="button" className="btn-primary" onClick={onChooseFiles} disabled={busy}>
          Choose images
        </button>
        <button type="button" className="btn-secondary" onClick={onChooseFolder} disabled={busy}>
          Choose folder
        </button>
      </div>
    </section>
  );
}

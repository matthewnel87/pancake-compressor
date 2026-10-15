/**
 * Before/after comparison for one finished file (docs/23 state 5).
 *
 * Both previews are made in Rust from the upright image and scaled to fit
 * the same box, so they appear at matched size. If a preview cannot be
 * made, a plain message is shown instead; the result itself is unaffected.
 * Focus moves to the panel when it opens and Escape closes it.
 */
import { useEffect, useRef, useState } from "react";
import { requestPreview, type Preview } from "../lib/ipc";
import { formatBytes } from "../lib/text";

interface PreviewPanelProps {
  batchId: string;
  fileIndex: number;
  fileName: string;
  onClose: () => void;
}

type Side = { status: "loading" } | { status: "ready"; preview: Preview } | { status: "failed" };

function usePreview(batchId: string, fileIndex: number, side: "original" | "output"): Side {
  const [state, setState] = useState<Side>({ status: "loading" });
  useEffect(() => {
    let cancelled = false;
    requestPreview(batchId, fileIndex, side)
      .then((preview) => !cancelled && setState({ status: "ready", preview }))
      .catch(() => !cancelled && setState({ status: "failed" }));
    return () => {
      cancelled = true;
    };
  }, [batchId, fileIndex, side]);
  return state;
}

function Figure({ label, side }: { label: string; side: Side }) {
  return (
    <figure className="flex flex-1 flex-col gap-2">
      <div className="flex aspect-square items-center justify-center overflow-hidden rounded-card border border-line bg-surface">
        {side.status === "ready" && (
          <img src={side.preview.dataUrl} alt={`${label} image`} className="h-full w-full object-contain" />
        )}
        {side.status === "loading" && <span className="text-ink-muted">Making preview…</span>}
        {side.status === "failed" && <span className="px-4 text-center text-ink-muted">No preview available for this image.</span>}
      </div>
      <figcaption className="text-sm tabular-nums">
        <strong>{label}</strong>
        {side.status === "ready" &&
          ` · ${formatBytes(side.preview.bytes)} · ${side.preview.width} × ${side.preview.height}`}
      </figcaption>
    </figure>
  );
}

export function PreviewPanel({ batchId, fileIndex, fileName, onClose }: PreviewPanelProps) {
  const original = usePreview(batchId, fileIndex, "original");
  const output = usePreview(batchId, fileIndex, "output");
  const heading = useRef<HTMLHeadingElement>(null);

  useEffect(() => heading.current?.focus(), []);

  return (
    <section
      aria-labelledby="preview-title"
      className="rounded-card border border-line bg-surface-raised p-5"
      onKeyDown={(e) => e.key === "Escape" && onClose()}
    >
      <div className="flex items-center gap-3">
        <h2 id="preview-title" ref={heading} tabIndex={-1} className="flex-1 truncate text-lg font-semibold">
          Compare: {fileName}
        </h2>
        <button type="button" className="btn-secondary" onClick={onClose}>
          Close
        </button>
      </div>
      <div className="mt-4 flex flex-col gap-4 sm:flex-row">
        <Figure label="Original" side={original} />
        <Figure label="Compressed" side={output} />
      </div>
    </section>
  );
}

/**
 * Format, preset and output-folder choices, plus the Start button.
 * Native <select> elements keep keyboard and screen-reader behaviour
 * familiar on both operating systems.
 */
import type { Preset, Target } from "../lib/ipc";
import { PRESET_OPTIONS, TARGET_OPTIONS } from "../lib/options";

interface ControlsProps {
  target: Target;
  preset: Preset;
  outputLabel: string;
  disabled: boolean;
  onTarget: (target: Target) => void;
  onPreset: (preset: Preset) => void;
  onChooseOutput: () => void;
  onStart: () => void;
}

export function Controls(props: ControlsProps) {
  const { target, preset, outputLabel, disabled } = props;
  return (
    <div className="flex flex-wrap items-end gap-4">
      <label className="flex flex-col gap-1 text-sm font-medium">
        Format
        <select
          className="control"
          value={target}
          disabled={disabled}
          onChange={(e) => props.onTarget(e.target.value as Target)}
        >
          {TARGET_OPTIONS.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
      </label>
      <label className="flex flex-col gap-1 text-sm font-medium">
        Quality
        <select
          className="control"
          value={preset}
          disabled={disabled}
          onChange={(e) => props.onPreset(e.target.value as Preset)}
        >
          {PRESET_OPTIONS.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
      </label>
      <div className="flex flex-col gap-1 text-sm font-medium">
        <span id="output-label">Save to</span>
        <button
          type="button"
          className="btn-secondary"
          aria-describedby="output-label"
          disabled={disabled}
          onClick={props.onChooseOutput}
        >
          {outputLabel}
        </button>
      </div>
      <button type="button" className="btn-primary ml-auto" disabled={disabled} onClick={props.onStart}>
        Compress images
      </button>
    </div>
  );
}

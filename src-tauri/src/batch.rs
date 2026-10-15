//! Bridge between the web UI and `pancake-core`'s scanner and batch engine.
//!
//! **Trust boundary (AGENTS.md rule 5, docs/03):** the web UI never supplies
//! a file path that Rust trusts. Native file and folder dialogs are opened
//! from Rust; Rust scans what the user picked and gives the UI an opaque
//! token plus counts. Starting a batch takes tokens, never paths. Events sent
//! back contain paths relative to the selected folder only, never full paths
//! (AGENTS.md rule 10).
//!
//! Tokens are the process id plus a counter: unique, but not secret. They
//! carry no authority beyond "files the user picked in this session", so a
//! guessed token can only reach files the user already chose.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use pancake_core::auto::{Consent, Preset, Target};
use pancake_core::batch::{
    self, BatchSettings, CancelToken, Event, Finished, Phase, Summary, Update,
};
use pancake_core::output::ConflictPolicy;
use pancake_core::scan::{OUTPUT_FOLDER_NAME, ScanResult, scan};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

/// Name of the event carrying [`BatchEventDto`] payloads.
pub const BATCH_EVENT: &str = "batch-event";

/// Everything the bridge remembers between commands.
#[derive(Default)]
pub struct AppState {
    next_id: AtomicU64,
    inputs: Mutex<HashMap<String, Selection>>,
    outputs: Mutex<HashMap<String, PathBuf>>,
    batches: Mutex<HashMap<String, Running>>,
}

struct Selection {
    roots: Vec<PathBuf>,
    scan: ScanResult,
}

struct Running {
    cancel: CancelToken,
    snapshot: Arc<Mutex<SnapshotDto>>,
}

impl AppState {
    /// A new identifier. The process id makes tokens from different
    /// sessions differ; uniqueness comes from the counter.
    fn new_id(&self, kind: &str) -> String {
        let n = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let salt = std::process::id();
        format!("{kind}-{salt:x}-{n}")
    }
}

/// Summary of a user's input selection, safe to show in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSelection {
    pub token: String,
    /// Name of the first selected item, for display ("Holiday photos").
    pub label: String,
    pub file_count: usize,
    pub skipped_count: usize,
    pub total_bytes: u64,
    pub truncated: bool,
}

/// A chosen output folder, safe to show in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSelection {
    pub token: String,
    pub label: String,
}

/// Options for [`start_batch`]. Unknown values are rejected by serde, so the
/// UI cannot invent settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BatchOptionsDto {
    pub input_token: String,
    pub output_token: Option<String>,
    pub target: TargetDto,
    pub preset: PresetDto,
    pub conflict: ConflictDto,
    #[serde(default)]
    pub allow_colour_profile_loss: bool,
    #[serde(default)]
    pub try_avif_in_auto: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TargetDto {
    Keep,
    Png,
    Jpeg,
    Webp,
    Avif,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PresetDto {
    Balanced,
    Smaller,
    BestQuality,
    Lossless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictDto {
    Rename,
    Skip,
}

/// One progress event as sent to the UI (schema version 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchEventDto {
    pub schema_version: u32,
    pub batch_id: String,
    pub sequence: u64,
    pub file_index: usize,
    /// Path relative to the selected folder, for display only.
    pub file: String,
    /// `decoding`, `encoding`, `writing`, `completed`, `notSmaller`,
    /// `skipped`, `failed` or `cancelled`.
    pub state: &'static str,
    /// Stable reason or error code, for final states.
    pub code: Option<&'static str>,
    pub original_bytes: Option<u64>,
    pub output_bytes: Option<u64>,
    pub output_format: Option<&'static str>,
    /// Visible changes the UI must mention, e.g. `transparencyFlattened`.
    pub notices: Vec<&'static str>,
    pub processed: usize,
    pub total: usize,
}

/// Current state of a batch, for catching up after missed events.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDto {
    pub batch_id: String,
    pub total: usize,
    pub processed: usize,
    pub last_sequence: u64,
    pub finished: bool,
    pub completed: usize,
    pub not_smaller: usize,
    pub skipped: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub original_bytes: u64,
    pub output_bytes: u64,
}

/// Display name of a path's last component. Never the full path.
fn label_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Scans picked paths and stores them under a new token.
pub fn register_selection(state: &AppState, roots: Vec<PathBuf>) -> InputSelection {
    let result = scan(&roots, None);
    let selection = InputSelection {
        token: state.new_id("in"),
        label: roots.first().map(|r| label_of(r)).unwrap_or_default(),
        file_count: result.files.len(),
        skipped_count: result.skipped.len(),
        total_bytes: result.files.iter().map(|f| f.size).sum(),
        truncated: result.truncated,
    };
    if let Ok(mut inputs) = state.inputs.lock() {
        inputs.insert(
            selection.token.clone(),
            Selection {
                roots,
                scan: result,
            },
        );
    }
    selection
}

/// Default output folder: `_pancake` inside the first selected folder, or
/// next to the first selected file (docs/08 output policy 1).
pub fn default_output_for(roots: &[PathBuf]) -> Option<PathBuf> {
    let first = roots.first()?;
    if first.is_dir() {
        Some(first.join(OUTPUT_FOLDER_NAME))
    } else {
        first.parent().map(|parent| parent.join(OUTPUT_FOLDER_NAME))
    }
}

fn settings_from(options: &BatchOptionsDto, output_root: PathBuf) -> BatchSettings {
    let mut settings = BatchSettings::new(output_root);
    settings.target = match options.target {
        TargetDto::Keep => Target::Keep,
        TargetDto::Png => Target::Png,
        TargetDto::Jpeg => Target::Jpeg,
        TargetDto::Webp => Target::WebP,
        TargetDto::Avif => Target::Avif,
        TargetDto::Auto => Target::Auto,
    };
    settings.preset = match options.preset {
        PresetDto::Balanced => Preset::Balanced,
        PresetDto::Smaller => Preset::Smaller,
        PresetDto::BestQuality => Preset::BestQuality,
        PresetDto::Lossless => Preset::Lossless,
    };
    settings.conflict = match options.conflict {
        ConflictDto::Rename => ConflictPolicy::Rename,
        ConflictDto::Skip => ConflictPolicy::Skip,
    };
    settings.consent = Consent {
        allow_colour_profile_loss: options.allow_colour_profile_loss,
        try_avif_in_auto: options.try_avif_in_auto,
    };
    settings
}

/// Converts a core event into the UI payload. `relative` is the file's path
/// relative to its selected root.
pub fn event_dto(batch_id: &str, event: &Event, relative: &Path) -> BatchEventDto {
    let mut dto = BatchEventDto {
        schema_version: event.schema_version,
        batch_id: batch_id.to_owned(),
        sequence: event.sequence,
        file_index: event.file_index,
        file: relative.to_string_lossy().replace('\\', "/"),
        state: "",
        code: None,
        original_bytes: None,
        output_bytes: None,
        output_format: None,
        notices: Vec::new(),
        processed: event.processed,
        total: event.total,
    };
    match &event.update {
        Update::Phase(Phase::Decoding) => dto.state = "decoding",
        Update::Phase(Phase::Encoding) => dto.state = "encoding",
        Update::Phase(Phase::Writing) => dto.state = "writing",
        Update::Finished(Finished::Completed {
            output_format,
            original_bytes,
            output_bytes,
            adjustments,
            reason,
            ..
        }) => {
            dto.state = "completed";
            dto.code = Some(reason);
            dto.original_bytes = Some(*original_bytes);
            dto.output_bytes = Some(*output_bytes);
            dto.output_format = Some(output_format.as_str());
            let flags = [
                (adjustments.orientation_applied, "orientationApplied"),
                (adjustments.transparency_flattened, "transparencyFlattened"),
                (adjustments.precision_reduced, "precisionReduced"),
                (adjustments.colour_profile_dropped, "colourProfileDropped"),
                (adjustments.metadata_dropped, "metadataDropped"),
            ];
            dto.notices = flags
                .into_iter()
                .filter(|(on, _)| *on)
                .map(|(_, name)| name)
                .collect();
        }
        Update::Finished(Finished::NotSmaller {
            original_bytes,
            reason,
        }) => {
            dto.state = "notSmaller";
            dto.code = Some(reason);
            dto.original_bytes = Some(*original_bytes);
        }
        Update::Finished(Finished::Skipped { code }) => {
            dto.state = "skipped";
            dto.code = Some(code);
        }
        Update::Finished(Finished::Failed { code }) => {
            dto.state = "failed";
            dto.code = Some(code);
        }
        Update::Finished(Finished::Cancelled) => dto.state = "cancelled",
    }
    dto
}

fn apply_summary(snapshot: &mut SnapshotDto, summary: &Summary) {
    snapshot.finished = true;
    snapshot.processed = summary.total;
    snapshot.completed = summary.completed;
    snapshot.not_smaller = summary.not_smaller;
    snapshot.skipped = summary.skipped;
    snapshot.failed = summary.failed;
    snapshot.cancelled = summary.cancelled;
    snapshot.original_bytes = summary.original_bytes;
    snapshot.output_bytes = summary.output_bytes;
}

/// Opens the native picker for files (`"files"`) or folders (`"folders"`)
/// and registers what the user chose. `None` if the dialog was cancelled.
#[tauri::command]
pub async fn select_inputs(
    app: AppHandle,
    state: State<'_, AppState>,
    kind: String,
) -> Result<Option<InputSelection>, String> {
    let dialog = app.dialog().file().set_title("Choose images to compress");
    let picked = match kind.as_str() {
        "files" => dialog
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "jpe", "jfif", "webp", "avif"],
            )
            .blocking_pick_files(),
        "folders" => dialog.blocking_pick_folders(),
        _ => return Err("invalid-selection-kind".into()),
    };
    let Some(picked) = picked else {
        return Ok(None);
    };
    let roots: Vec<PathBuf> = picked
        .into_iter()
        .filter_map(|p| p.into_path().ok())
        .collect();
    if roots.is_empty() {
        return Ok(None);
    }
    Ok(Some(register_selection(&state, roots)))
}

/// Opens the native folder picker for the output folder.
#[tauri::command]
pub async fn select_output_folder(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<OutputSelection>, String> {
    let picked = app
        .dialog()
        .file()
        .set_title("Choose where to save compressed images")
        .set_can_create_directories(true)
        .blocking_pick_folder();
    let Some(path) = picked.and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    let selection = OutputSelection {
        token: state.new_id("out"),
        label: label_of(&path),
    };
    if let Ok(mut outputs) = state.outputs.lock() {
        outputs.insert(selection.token.clone(), path);
    }
    Ok(Some(selection))
}

/// Starts a batch in the background and returns its id. Progress arrives
/// as [`BATCH_EVENT`] events.
#[tauri::command]
pub fn start_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    options: BatchOptionsDto,
) -> Result<String, String> {
    let (roots, files) = {
        let inputs = state.inputs.lock().map_err(|_| "state-unavailable")?;
        let selection = inputs
            .get(&options.input_token)
            .ok_or("unknown-input-token")?;
        (selection.roots.clone(), selection.scan.files.clone())
    };
    let output_root = match &options.output_token {
        Some(token) => state
            .outputs
            .lock()
            .map_err(|_| "state-unavailable")?
            .get(token)
            .cloned()
            .ok_or("unknown-output-token")?,
        None => default_output_for(&roots).ok_or("no-output-folder")?,
    };
    // Files inside the output folder (for example from an earlier run into a
    // custom folder) must not be processed again.
    let output_canonical = std::fs::canonicalize(&output_root).ok();
    let files: Vec<_> = files
        .into_iter()
        .filter(|f| {
            output_canonical
                .as_ref()
                .is_none_or(|out| !f.path.starts_with(out))
        })
        .collect();

    let batch_id = state.new_id("batch");
    let cancel = CancelToken::new();
    let snapshot = Arc::new(Mutex::new(SnapshotDto {
        batch_id: batch_id.clone(),
        total: files.len(),
        ..SnapshotDto::default()
    }));
    state
        .batches
        .lock()
        .map_err(|_| "state-unavailable")?
        .insert(
            batch_id.clone(),
            Running {
                cancel: cancel.clone(),
                snapshot: Arc::clone(&snapshot),
            },
        );

    let settings = settings_from(&options, output_root);
    let id = batch_id.clone();
    std::thread::Builder::new()
        .name(format!("pancake-{id}"))
        .spawn(move || {
            let sink = |event: &Event| {
                let relative = files
                    .get(event.file_index)
                    .map(|f| f.relative.as_path())
                    .unwrap_or(Path::new(""));
                let dto = event_dto(&id, event, relative);
                if let Ok(mut snap) = snapshot.lock() {
                    snap.processed = event.processed;
                    snap.last_sequence = event.sequence;
                }
                // A closed window cannot receive events; the batch continues.
                let _ = app.emit(BATCH_EVENT, dto);
            };
            let summary = batch::run(&files, &settings, &cancel, &sink);
            if let Ok(mut snap) = snapshot.lock() {
                apply_summary(&mut snap, &summary);
                let _ = app.emit("batch-finished", snap.clone());
            }
        })
        .map_err(|_| "could-not-start-batch")?;
    Ok(batch_id)
}

/// Asks a running batch to stop. Finished outputs are kept.
#[tauri::command]
pub fn cancel_batch(state: State<'_, AppState>, batch_id: String) -> Result<(), String> {
    let batches = state.batches.lock().map_err(|_| "state-unavailable")?;
    let running = batches.get(&batch_id).ok_or("unknown-batch")?;
    running.cancel.cancel();
    Ok(())
}

/// Returns the latest known state of a batch.
#[tauri::command]
pub fn get_batch_snapshot(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<SnapshotDto, String> {
    let batches = state.batches.lock().map_err(|_| "state-unavailable")?;
    let running = batches.get(&batch_id).ok_or("unknown-batch")?;
    let snapshot = running.snapshot.lock().map_err(|_| "state-unavailable")?;
    Ok(snapshot.clone())
}

#[cfg(test)]
mod tests;

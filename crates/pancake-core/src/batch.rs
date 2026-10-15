//! The batch engine: process many files with bounded CPU and memory, report
//! progress as ordered events, and cancel cleanly (docs/03, docs/08, M2).
//!
//! **Bounded work, not one thread per file.** A fixed number of worker
//! threads ([`BatchSettings::workers`]) pull files from a shared index.
//! Before decoding, each file reserves its estimated peak memory from a
//! budget ([`BatchSettings::memory_budget`]); if the budget is in use it
//! waits, and a file that could never fit is skipped with a reason.
//!
//! **Estimate:** [`PEAK_BYTES_PER_PIXEL`] comes from the M1 benchmark
//! (about 1 GB peak for a 12-megapixel image, ~84 bytes per pixel, across
//! decode, orientation copy, RGBA conversion, encoding and verification).
//! It is deliberately pessimistic until those copies are reduced.
//!
//! **Events.** Every file produces phase updates and exactly one final
//! result. Events are delivered one at a time under a lock, in strictly
//! increasing [`Event::sequence`] order, and `processed` counts final
//! results only, so progress = processed / total is a real measure
//! (AGENTS.md rule 6). No per-file percentage is invented.
//!
//! **Cancellation** is checked before each stage and while waiting for
//! memory. A stage already running in a codec finishes first (codecs cannot
//! be interrupted). Files finished before cancellation keep their outputs;
//! files not finished get a `Cancelled` result and write nothing.
//!
//! **Panics** inside a codec are caught per file and reported as
//! `internal-error` instead of ending the batch (ADR-030).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::auto::{Consent, Preset, Target, compress};
use crate::decode::{DecodeError, decode};
use crate::encode::{Adjustments, EncodeError};
use crate::format::{ImageFormat, InspectError, preflight};
use crate::limits::ImageLimits;
use crate::output::{
    ConflictPolicy, OutputError, WriteOutcome, destination, extension_for, write_new,
};
use crate::scan::ScannedFile;

/// Version of the [`Event`] layout. Bump when fields change meaning.
pub const EVENT_SCHEMA_VERSION: u32 = 1;

/// Pessimistic peak working memory per pixel (see module docs).
pub const PEAK_BYTES_PER_PIXEL: u64 = 88;

/// Fixed per-file overhead added to every memory estimate.
const PER_FILE_OVERHEAD: u64 = 64 * 1024 * 1024;

/// Largest input file read into memory. Bigger files are skipped.
pub const MAX_INPUT_BYTES: u64 = 512 * 1024 * 1024;

/// How often a worker waiting for memory re-checks cancellation.
const CANCEL_POLL: Duration = Duration::from_millis(100);

/// Settings for one batch.
#[derive(Debug, Clone)]
pub struct BatchSettings {
    pub target: Target,
    pub preset: Preset,
    pub consent: Consent,
    pub conflict: ConflictPolicy,
    /// Folder that receives outputs, recreating each file's relative path.
    pub output_root: PathBuf,
    pub limits: ImageLimits,
    /// Total bytes of estimated working memory allowed at once.
    pub memory_budget: u64,
    /// Number of worker threads (at least 1).
    pub workers: usize,
}

impl BatchSettings {
    /// Defaults: Smart Auto off (keep format), Balanced, numbered renames,
    /// a 4 GiB memory budget and one worker per CPU (max 8). The desktop app
    /// should lower the budget on machines with little RAM.
    pub fn new(output_root: PathBuf) -> Self {
        let cpus = std::thread::available_parallelism().map_or(2, |n| n.get());
        Self {
            target: Target::Keep,
            preset: Preset::Balanced,
            consent: Consent::default(),
            conflict: ConflictPolicy::Rename,
            output_root,
            limits: ImageLimits::default(),
            memory_budget: 4 * 1024 * 1024 * 1024,
            workers: cpus.clamp(1, 8),
        }
    }
}

/// A shared flag that asks a running batch to stop.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// Non-final stages a file passes through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Decoding,
    Encoding,
    Writing,
}

/// The one final result every file gets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finished {
    /// A new, smaller file was written.
    Completed {
        output: PathBuf,
        output_format: ImageFormat,
        original_bytes: u64,
        output_bytes: u64,
        adjustments: Adjustments,
        reason: &'static str,
    },
    /// Nothing smaller was possible; no file written (ADR-015).
    NotSmaller {
        original_bytes: u64,
        reason: &'static str,
    },
    /// Not processed for an expected reason (not an image, animated, needs
    /// consent, output exists...).
    Skipped { code: &'static str },
    /// Processing failed (damaged file, disk error...).
    Failed { code: &'static str },
    /// The batch was cancelled before this file finished.
    Cancelled,
}

/// What changed for one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Update {
    Phase(Phase),
    Finished(Finished),
}

/// One progress event. See module docs for ordering guarantees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub schema_version: u32,
    /// Strictly increasing within a batch, starting at 1.
    pub sequence: u64,
    /// Index into the file list given to [`run`].
    pub file_index: usize,
    pub update: Update,
    /// Files with a final result so far.
    pub processed: usize,
    pub total: usize,
}

/// Totals for a finished batch. Always `completed + not_smaller + skipped +
/// failed + cancelled == total`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Summary {
    pub total: usize,
    pub completed: usize,
    pub not_smaller: usize,
    pub skipped: usize,
    pub failed: usize,
    pub cancelled: usize,
    /// Sum of original sizes of completed files.
    pub original_bytes: u64,
    /// Sum of output sizes of completed files.
    pub output_bytes: u64,
}

/// Serialises event delivery and keeps the counters consistent.
struct Reporter<'a> {
    state: Mutex<(u64, usize, Summary)>, // (sequence, processed, summary)
    total: usize,
    sink: &'a (dyn Fn(&Event) + Sync),
}

impl Reporter<'_> {
    fn send(&self, file_index: usize, update: Update) {
        let Ok(mut guard) = self.state.lock() else {
            return; // a panicking sink poisoned the lock; stop reporting
        };
        let (sequence, processed, summary) = &mut *guard;
        *sequence += 1;
        if let Update::Finished(result) = &update {
            *processed += 1;
            match result {
                Finished::Completed {
                    original_bytes,
                    output_bytes,
                    ..
                } => {
                    summary.completed += 1;
                    summary.original_bytes += original_bytes;
                    summary.output_bytes += output_bytes;
                }
                Finished::NotSmaller { .. } => summary.not_smaller += 1,
                Finished::Skipped { .. } => summary.skipped += 1,
                Finished::Failed { .. } => summary.failed += 1,
                Finished::Cancelled => summary.cancelled += 1,
            }
        }
        (self.sink)(&Event {
            schema_version: EVENT_SCHEMA_VERSION,
            sequence: *sequence,
            file_index,
            update,
            processed: *processed,
            total: self.total,
        });
    }
}

/// Weighted semaphore over the memory budget.
struct MemoryGate {
    available: Mutex<u64>,
    released: Condvar,
}

struct Reservation<'a> {
    gate: &'a MemoryGate,
    bytes: u64,
}

impl Drop for Reservation<'_> {
    fn drop(&mut self) {
        if let Ok(mut available) = self.gate.available.lock() {
            *available += self.bytes;
        }
        self.gate.released.notify_all();
    }
}

impl MemoryGate {
    /// Waits until `bytes` are free, or returns `None` if cancelled.
    fn reserve(&self, bytes: u64, cancel: &CancelToken) -> Option<Reservation<'_>> {
        let mut available = self.available.lock().ok()?;
        while *available < bytes {
            if cancel.is_cancelled() {
                return None;
            }
            available = self.released.wait_timeout(available, CANCEL_POLL).ok()?.0;
        }
        *available -= bytes;
        Some(Reservation { gate: self, bytes })
    }
}

/// Estimated peak working memory for an image of this size.
pub fn estimate_memory(width: u32, height: u32, file_bytes: u64) -> u64 {
    (u64::from(width) * u64::from(height))
        .saturating_mul(PEAK_BYTES_PER_PIXEL)
        .saturating_add(file_bytes.saturating_mul(3))
        .saturating_add(PER_FILE_OVERHEAD)
}

fn inspect_result(error: InspectError) -> Finished {
    let code = error.code();
    match error {
        InspectError::Truncated(_)
        | InspectError::Malformed(..)
        | InspectError::ZeroDimensions(_) => Finished::Failed { code },
        _ => Finished::Skipped { code },
    }
}

fn decode_result(error: DecodeError) -> Finished {
    match error {
        DecodeError::Preflight(inner) => inspect_result(inner),
        DecodeError::UnsupportedFeature(_) => Finished::Skipped { code: error.code() },
        _ => Finished::Failed { code: error.code() },
    }
}

fn encode_result(error: &EncodeError) -> Finished {
    let code = error.code();
    match error {
        EncodeError::QualityNotReached(_)
        | EncodeError::ColourProfileNotSupported(_)
        | EncodeError::ProtectedContent(_) => Finished::Skipped { code },
        _ => Finished::Failed { code },
    }
}

fn output_result(error: &OutputError) -> Finished {
    Finished::Failed { code: error.code() }
}

/// Processes one file through every stage. Returns its final result.
fn process(
    file: &ScannedFile,
    index: usize,
    settings: &BatchSettings,
    output_root: &Path,
    gate: &MemoryGate,
    cancel: &CancelToken,
    reporter: &Reporter<'_>,
) -> Finished {
    if cancel.is_cancelled() {
        return Finished::Cancelled;
    }
    if file.size > MAX_INPUT_BYTES {
        return Finished::Skipped {
            code: "file-too-large",
        };
    }
    let Ok(bytes) = std::fs::read(&file.path) else {
        return Finished::Failed {
            code: "input-unreadable",
        };
    };
    let info = match preflight(&bytes, &settings.limits) {
        Ok(info) => info,
        Err(error) => return inspect_result(error),
    };
    let needed = estimate_memory(info.width, info.height, bytes.len() as u64);
    if needed > settings.memory_budget {
        return Finished::Skipped {
            code: "memory-budget-exceeded",
        };
    }
    let Some(_reservation) = gate.reserve(needed, cancel) else {
        return Finished::Cancelled;
    };

    reporter.send(index, Update::Phase(Phase::Decoding));
    let decoded = match decode(&bytes, &settings.limits) {
        Ok(decoded) => decoded,
        Err(error) => return decode_result(error),
    };
    if cancel.is_cancelled() {
        return Finished::Cancelled;
    }

    reporter.send(index, Update::Phase(Phase::Encoding));
    let outcome = match compress(
        &bytes,
        &decoded,
        settings.target,
        settings.preset,
        settings.consent,
    ) {
        Ok(outcome) => outcome,
        Err(error) => return encode_result(&error),
    };
    drop(decoded);
    let original_bytes = bytes.len() as u64;
    if outcome.kept_original {
        return Finished::NotSmaller {
            original_bytes,
            reason: outcome.reason.code(),
        };
    }
    if cancel.is_cancelled() {
        return Finished::Cancelled;
    }

    reporter.send(index, Update::Phase(Phase::Writing));
    let (folder, stem) = match destination(output_root, &file.relative) {
        Ok(place) => place,
        Err(error) => return output_result(&error),
    };
    let extension = extension_for(outcome.output_format);
    match write_new(&folder, &stem, extension, &outcome.bytes, settings.conflict) {
        Ok(WriteOutcome::Written(output)) => Finished::Completed {
            output,
            output_format: outcome.output_format,
            original_bytes,
            output_bytes: outcome.bytes.len() as u64,
            adjustments: outcome.adjustments,
            reason: outcome.reason.code(),
        },
        Ok(WriteOutcome::SkippedExisting(_)) => Finished::Skipped {
            code: "output-exists",
        },
        Err(error) => output_result(&error),
    }
}

/// Checks the output folder before any work: it is created if missing, must
/// not be a link, and is returned in canonical form.
pub fn prepare_output_root(output_root: &Path) -> Result<PathBuf, OutputError> {
    std::fs::create_dir_all(output_root)?;
    let metadata = std::fs::symlink_metadata(output_root)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(OutputError::UnsafePath);
    }
    Ok(std::fs::canonicalize(output_root)?)
}

/// Runs a batch to completion (or cancellation) and returns its totals.
///
/// Blocks the calling thread; the desktop app calls it from a background
/// thread. `on_event` is called from worker threads, one event at a time.
pub fn run(
    files: &[ScannedFile],
    settings: &BatchSettings,
    cancel: &CancelToken,
    on_event: &(dyn Fn(&Event) + Sync),
) -> Summary {
    let reporter = Reporter {
        state: Mutex::new((0, 0, Summary::default())),
        total: files.len(),
        sink: on_event,
    };
    let output_root = prepare_output_root(&settings.output_root);
    let gate = MemoryGate {
        available: Mutex::new(settings.memory_budget),
        released: Condvar::new(),
    };
    let next = AtomicUsize::new(0);
    let workers = settings.workers.clamp(1, files.len().max(1));

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::SeqCst);
                    let Some(file) = files.get(index) else {
                        break;
                    };
                    let result = match &output_root {
                        Err(error) => output_result(error),
                        Ok(root) => catch_unwind(AssertUnwindSafe(|| {
                            process(file, index, settings, root, &gate, cancel, &reporter)
                        }))
                        .unwrap_or(Finished::Failed {
                            code: "internal-error",
                        }),
                    };
                    reporter.send(index, Update::Finished(result));
                }
            });
        }
    });

    let mut summary = reporter
        .state
        .into_inner()
        .map(|(_, _, summary)| summary)
        .unwrap_or_default();
    summary.total = files.len();
    summary
}

#[cfg(test)]
mod tests;

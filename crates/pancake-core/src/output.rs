//! Writing results to disk without ever overwriting or leaving partial files.
//!
//! Invariants (docs/08 "Output policy", ADR-006):
//! - **Never overwrite.** The final name is claimed with a no-clobber rename
//!   (`renameat` with `RENAME_NOREPLACE`/`RENAME_EXCL` on macOS and Linux,
//!   `MoveFileExW` without `MOVEFILE_REPLACE_EXISTING` on Windows, via the
//!   `tempfile` crate). If the name exists, the next numbered name is tried
//!   (`photo-pancake-2.webp`, ...) or the file is skipped, as the user chose.
//!   Because the check and the claim are one operation, two jobs finishing at
//!   once cannot take the same name.
//! - **No partial files under a final name.** Bytes go to a hidden temporary
//!   file (`.pancake-*.tmp`) in the destination folder, are flushed to disk,
//!   and only then renamed into place. On any error the temporary file is
//!   deleted when it is dropped. Same-folder temporaries keep the rename on
//!   one filesystem.
//! - **Relative paths cannot escape.** A source's path relative to its
//!   selected root must consist of ordinary names only: no `..`, no absolute
//!   or drive prefixes, no Windows device names. Checked here even though the
//!   scanner (M2) produces these paths, because this is the last step before
//!   touching the disk.
//!
//! **Not owned here:** choosing the output root, checking it is not inside
//! an input tree, and refusing symlinked destination folders. Those are
//! batch-level checks (M2 scanner/batch modules).

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use crate::format::ImageFormat;

/// What to do when the destination name already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictPolicy {
    /// Use the next free name: `name-pancake-2.ext`, `name-pancake-3.ext`...
    Rename,
    /// Leave the existing file alone and report the file as skipped.
    Skip,
}

/// Highest numbered suffix tried before giving up.
const MAX_RENAME_ATTEMPTS: u32 = 9_999;

/// Result of a successful [`write_new`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteOutcome {
    /// Written to this path (which did not exist before).
    Written(PathBuf),
    /// The destination existed and the policy was [`ConflictPolicy::Skip`].
    SkippedExisting(PathBuf),
}

/// Why a write failed. Messages never include file contents.
#[derive(Debug)]
pub enum OutputError {
    /// The relative path contains `..`, a root, a drive prefix or a device
    /// name, so it could point outside the output folder.
    UnsafePath,
    /// Every numbered name up to the limit is taken.
    NoFreeName,
    /// The operating system refused (permissions, disk full, path too long,
    /// read-only volume...). The original error kind is kept for the UI.
    Io(io::ErrorKind),
}

impl OutputError {
    /// Stable machine-readable code for progress events.
    pub fn code(&self) -> &'static str {
        match self {
            OutputError::UnsafePath => "unsafe-output-path",
            OutputError::NoFreeName => "no-free-output-name",
            OutputError::Io(io::ErrorKind::PermissionDenied) => "output-permission-denied",
            OutputError::Io(io::ErrorKind::StorageFull) => "output-disk-full",
            OutputError::Io(io::ErrorKind::ReadOnlyFilesystem) => "output-read-only",
            OutputError::Io(_) => "output-write-failed",
        }
    }
}

impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            OutputError::UnsafePath => {
                "This file's location cannot be recreated safely in the output folder."
            }
            OutputError::NoFreeName => {
                "Too many files with this name already exist in the output folder."
            }
            OutputError::Io(io::ErrorKind::PermissionDenied) => {
                "Pancake Compressor is not allowed to write to the output folder."
            }
            OutputError::Io(io::ErrorKind::StorageFull) => "The disk is full.",
            OutputError::Io(io::ErrorKind::ReadOnlyFilesystem) => "The output folder is read-only.",
            OutputError::Io(_) => "The file could not be saved.",
        })
    }
}

impl std::error::Error for OutputError {}

impl From<io::Error> for OutputError {
    fn from(error: io::Error) -> Self {
        OutputError::Io(error.kind())
    }
}

/// File name extension written for each output format.
pub fn extension_for(output_format: ImageFormat) -> &'static str {
    match output_format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::WebP => "webp",
        ImageFormat::Avif => "avif",
    }
}

/// Windows device names, refused on every OS so output trees are portable.
fn is_device_name(name: &OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    // "CON", "con.txt" and "con .txt" all refer to the device on Windows.
    let stem = name
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end()
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem
                .as_bytes()
                .get(3)
                .is_some_and(|d| (b'1'..=b'9').contains(d)))
}

/// Checks that `relative` is made only of ordinary names (no `..`, root,
/// drive prefix, `.` or device names).
pub fn is_safe_relative(relative: &Path) -> bool {
    let mut any = false;
    for component in relative.components() {
        match component {
            Component::Normal(name) if !is_device_name(name) => any = true,
            _ => return false,
        }
    }
    any
}

/// Where the output for `relative_source` (a path relative to its selected
/// root, e.g. `holiday/beach.png`) goes under `output_root`: the same
/// relative folder, the source's file stem, and the output's extension.
///
/// Returns the destination folder and the file stem. The extension is added
/// by [`write_new`].
pub fn destination(
    output_root: &Path,
    relative_source: &Path,
) -> Result<(PathBuf, OsString), OutputError> {
    if !is_safe_relative(relative_source) {
        return Err(OutputError::UnsafePath);
    }
    let stem = relative_source
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .ok_or(OutputError::UnsafePath)?
        .to_os_string();
    let folder = match relative_source.parent() {
        Some(parent) => output_root.join(parent),
        None => output_root.to_path_buf(),
    };
    Ok((folder, stem))
}

fn numbered(stem: &OsStr, attempt: u32, extension: &str) -> OsString {
    let mut name = stem.to_os_string();
    if attempt > 1 {
        name.push(format!("-pancake-{attempt}"));
    }
    name.push(".");
    name.push(extension);
    name
}

/// Writes `bytes` to `folder/stem.extension` without overwriting anything.
///
/// Creates `folder` if needed. The temporary file is flushed to disk before
/// it is renamed, so a crash leaves either nothing or a complete file, never
/// a partial file under the final name.
pub fn write_new(
    folder: &Path,
    stem: &OsStr,
    extension: &str,
    bytes: &[u8],
    policy: ConflictPolicy,
) -> Result<WriteOutcome, OutputError> {
    if stem.is_empty() || is_device_name(stem) || !is_safe_relative(Path::new(stem)) {
        return Err(OutputError::UnsafePath);
    }
    std::fs::create_dir_all(folder)?;

    let mut temp = tempfile::Builder::new()
        .prefix(".pancake-")
        .suffix(".tmp")
        .tempfile_in(folder)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;

    for attempt in 1..=MAX_RENAME_ATTEMPTS {
        let target = folder.join(numbered(stem, attempt, extension));
        match temp.persist_noclobber(&target) {
            Ok(_) => return Ok(WriteOutcome::Written(target)),
            Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => {
                if policy == ConflictPolicy::Skip {
                    // `error.file` is dropped here, deleting the temporary.
                    return Ok(WriteOutcome::SkippedExisting(target));
                }
                temp = error.file;
            }
            Err(error) => return Err(error.error.into()),
        }
    }
    Err(OutputError::NoFreeName)
}

#[cfg(test)]
mod tests;

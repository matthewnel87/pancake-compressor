//! Listing the files inside the folders (or single files) a user selected.
//!
//! Rules (docs/08 "Security before processing"):
//! - Only the selected roots are walked. Roots are canonicalised once; every
//!   file's path is then the root plus ordinary names, so its path relative
//!   to the root can be recreated under the output folder.
//! - **Links are never followed.** Symlinks, and on Windows any reparse
//!   point (junctions, mount points), are skipped and reported. This stops
//!   a link from pulling in files outside the selection or creating a loop.
//! - Only regular files are returned. Devices, pipes and sockets are skipped.
//! - The output folder, and any folder named `_pancake`, is never entered, so
//!   results are not fed back in as inputs.
//! - A root inside another selected root is skipped as already included, so
//!   no file is processed twice.
//! - OS clutter (`.DS_Store`, `Thumbs.db`, `desktop.ini`, `._*` AppleDouble
//!   files) is ignored silently: it is never an image the user chose.
//! - Depth and file-count limits stop a pathological tree from running away.
//!
//! This module only lists files. Whether each one is an image, and whether
//! it can be processed, is decided per file by the batch engine.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Name of the default output folder; never scanned.
pub const OUTPUT_FOLDER_NAME: &str = "_pancake";

/// Deepest folder nesting walked below a root.
pub const MAX_DEPTH: usize = 64;

/// Most files listed in one scan. Larger selections are reported as
/// truncated rather than exhausting memory.
pub const MAX_FILES: usize = 100_000;

/// A regular file found under a selected root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedFile {
    /// Full path: canonical root joined with `relative`.
    pub path: PathBuf,
    /// Path relative to the selected root (for a selected single file, just
    /// its name). Always ordinary names only.
    pub relative: PathBuf,
    /// Size in bytes when listed.
    pub size: u64,
}

/// Something found during the scan that will not be processed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    /// Path relative to its root (or the root as given, for root problems).
    /// For display only; never used to open a file.
    pub relative: PathBuf,
    pub reason: SkipReason,
}

/// Why a scanned entry is not processed. Stable [`SkipReason::code`] values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// The selected item no longer exists or cannot be opened.
    Unreadable,
    /// A symlink, junction or other link; not followed.
    Link,
    /// A device, pipe, socket or other non-regular file.
    NotRegularFile,
    /// The output folder (or a `_pancake` folder).
    OutputFolder,
    /// This selected root is inside another selected root.
    AlreadyIncluded,
    /// Nested deeper than [`MAX_DEPTH`].
    TooDeep,
}

impl SkipReason {
    pub fn code(&self) -> &'static str {
        match self {
            SkipReason::Unreadable => "unreadable",
            SkipReason::Link => "link-not-followed",
            SkipReason::NotRegularFile => "not-a-regular-file",
            SkipReason::OutputFolder => "output-folder",
            SkipReason::AlreadyIncluded => "already-included",
            SkipReason::TooDeep => "folder-too-deep",
        }
    }
}

/// Everything a scan found.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ScanResult {
    pub files: Vec<ScannedFile>,
    pub skipped: Vec<Skipped>,
    /// `true` if [`MAX_FILES`] was reached and some files were not listed.
    pub truncated: bool,
}

fn is_os_clutter(name: &str) -> bool {
    name == ".DS_Store"
        || name.eq_ignore_ascii_case("thumbs.db")
        || name.eq_ignore_ascii_case("desktop.ini")
        || name.starts_with("._")
}

/// `true` for symlinks and, on Windows, any reparse point (junctions and
/// mount points included), checked without following the link.
fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return true;
        }
    }
    metadata.file_type().is_symlink()
}

struct Walker<'a> {
    output_root: Option<&'a Path>,
    result: ScanResult,
    seen: HashSet<PathBuf>,
}

impl Walker<'_> {
    fn is_output(&self, path: &Path) -> bool {
        self.output_root.is_some_and(|out| path == out)
            || path.file_name().is_some_and(|n| n == OUTPUT_FOLDER_NAME)
    }

    fn skip(&mut self, relative: PathBuf, reason: SkipReason) {
        self.result.skipped.push(Skipped { relative, reason });
    }

    fn add_file(&mut self, path: PathBuf, relative: PathBuf, size: u64) {
        if self.result.files.len() >= MAX_FILES {
            self.result.truncated = true;
            return;
        }
        if self.seen.insert(path.clone()) {
            self.result.files.push(ScannedFile {
                path,
                relative,
                size,
            });
        }
    }

    fn walk(&mut self, folder: &Path, relative: &Path, depth: usize) {
        if depth > MAX_DEPTH {
            self.skip(relative.to_path_buf(), SkipReason::TooDeep);
            return;
        }
        let entries = match fs::read_dir(folder) {
            Ok(entries) => entries,
            Err(_) => {
                self.skip(relative.to_path_buf(), SkipReason::Unreadable);
                return;
            }
        };
        // Sorted so results (and their order in the UI) are reproducible.
        let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            if self.result.truncated {
                return;
            }
            let name = entry.file_name();
            if name.to_str().is_some_and(is_os_clutter) {
                continue;
            }
            let path = entry.path();
            let child_relative = relative.join(&name);
            // `symlink_metadata` does not follow links.
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                self.skip(child_relative, SkipReason::Unreadable);
                continue;
            };
            if is_link(&metadata) {
                self.skip(child_relative, SkipReason::Link);
            } else if metadata.is_dir() {
                if self.is_output(&path) {
                    self.skip(child_relative, SkipReason::OutputFolder);
                } else {
                    self.walk(&path, &child_relative, depth + 1);
                }
            } else if metadata.is_file() {
                self.add_file(path, child_relative, metadata.len());
            } else {
                self.skip(child_relative, SkipReason::NotRegularFile);
            }
        }
    }
}

/// Lists the regular files under each selected root.
///
/// `output_root` (if already chosen) is excluded wherever it appears. Roots
/// that are links, unreadable, or inside another root are reported in
/// [`ScanResult::skipped`] with the path as given.
pub fn scan(roots: &[PathBuf], output_root: Option<&Path>) -> ScanResult {
    let canonical_output = output_root.and_then(|out| fs::canonicalize(out).ok());
    let mut walker = Walker {
        output_root: canonical_output.as_deref(),
        result: ScanResult::default(),
        seen: HashSet::new(),
    };

    // Resolve roots first so containment between them can be checked.
    let mut resolved: Vec<(PathBuf, PathBuf)> = Vec::new(); // (as given, canonical)
    for root in roots {
        let link = fs::symlink_metadata(root).map(|m| is_link(&m));
        match (link, fs::canonicalize(root)) {
            (Ok(true), _) => walker.skip(root.clone(), SkipReason::Link),
            (Ok(false), Ok(canonical)) => resolved.push((root.clone(), canonical)),
            _ => walker.skip(root.clone(), SkipReason::Unreadable),
        }
    }

    for (index, (given, root)) in resolved.iter().enumerate() {
        let inside_other = resolved.iter().enumerate().any(|(other, (_, candidate))| {
            other != index && root != candidate && root.starts_with(candidate)
        });
        let duplicate_root = resolved[..index].iter().any(|(_, earlier)| earlier == root);
        if inside_other || duplicate_root {
            walker.skip(given.clone(), SkipReason::AlreadyIncluded);
            continue;
        }
        if walker.is_output(root) {
            walker.skip(given.clone(), SkipReason::OutputFolder);
            continue;
        }
        match fs::symlink_metadata(root) {
            Ok(metadata) if metadata.is_dir() => walker.walk(root, Path::new(""), 0),
            Ok(metadata) if metadata.is_file() => {
                let name = root.file_name().map(PathBuf::from).unwrap_or_default();
                walker.add_file(root.clone(), name, metadata.len());
            }
            Ok(_) => walker.skip(given.clone(), SkipReason::NotRegularFile),
            Err(_) => walker.skip(given.clone(), SkipReason::Unreadable),
        }
    }
    walker.result
}

#[cfg(test)]
mod tests;

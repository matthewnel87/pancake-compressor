//! Output writer tests. Each test works in its own temporary folder.

use std::collections::HashSet;
use std::sync::{Arc, Barrier};

use super::*;

fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"))
}

fn leftovers(folder: &Path) -> Vec<String> {
    std::fs::read_dir(folder)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|name| name.starts_with(".pancake-"))
                .collect()
        })
        .unwrap_or_default()
}

fn write_ok(folder: &Path, stem: &str, bytes: &[u8], policy: ConflictPolicy) -> WriteOutcome {
    write_new(folder, OsStr::new(stem), "webp", bytes, policy)
        .unwrap_or_else(|error| unreachable!("{error:?}"))
}

#[test]
fn writes_complete_file_and_leaves_no_temporary() {
    let dir = temp_dir();
    let outcome = write_ok(dir.path(), "photo", b"hello", ConflictPolicy::Rename);
    let expected = dir.path().join("photo.webp");
    assert_eq!(outcome, WriteOutcome::Written(expected.clone()));
    assert_eq!(std::fs::read(expected).unwrap_or_default(), b"hello");
    assert!(leftovers(dir.path()).is_empty());
}

#[test]
fn existing_files_are_never_overwritten() {
    let dir = temp_dir();
    let existing = dir.path().join("photo.webp");
    std::fs::write(&existing, b"original").unwrap_or_else(|error| unreachable!("{error}"));

    let renamed = write_ok(dir.path(), "photo", b"new", ConflictPolicy::Rename);
    assert_eq!(
        renamed,
        WriteOutcome::Written(dir.path().join("photo-pancake-2.webp"))
    );
    let third = write_ok(dir.path(), "photo", b"newer", ConflictPolicy::Rename);
    assert_eq!(
        third,
        WriteOutcome::Written(dir.path().join("photo-pancake-3.webp"))
    );

    let skipped = write_ok(dir.path(), "photo", b"ignored", ConflictPolicy::Skip);
    assert_eq!(skipped, WriteOutcome::SkippedExisting(existing.clone()));

    assert_eq!(std::fs::read(existing).unwrap_or_default(), b"original");
    assert!(
        leftovers(dir.path()).is_empty(),
        "temporary files left behind"
    );
}

#[test]
fn concurrent_writers_never_share_a_name() {
    let dir = temp_dir();
    let threads = 50;
    let barrier = Arc::new(Barrier::new(threads));
    let handles: Vec<_> = (0..threads)
        .map(|i| {
            let folder = dir.path().to_path_buf();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait(); // start all writes at the same moment
                write_new(
                    &folder,
                    OsStr::new("same"),
                    "webp",
                    format!("{i}").as_bytes(),
                    ConflictPolicy::Rename,
                )
            })
        })
        .collect();
    let mut paths = HashSet::new();
    for handle in handles {
        match handle.join() {
            Ok(Ok(WriteOutcome::Written(path))) => {
                assert!(paths.insert(path), "two writers claimed the same name");
            }
            other => unreachable!("unexpected result {other:?}"),
        }
    }
    assert_eq!(paths.len(), threads);
    // Every file still holds exactly what one writer wrote.
    let contents: HashSet<String> = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap_or_default())
        .collect();
    assert_eq!(contents.len(), threads);
    assert!(leftovers(dir.path()).is_empty());
}

#[test]
fn nested_relative_paths_are_recreated() {
    let dir = temp_dir();
    let (folder, stem) = destination(dir.path(), Path::new("trips/2026/beach.png"))
        .unwrap_or_else(|error| unreachable!("{error:?}"));
    assert_eq!(folder, dir.path().join("trips/2026"));
    let outcome = write_new(&folder, &stem, "webp", b"x", ConflictPolicy::Rename)
        .unwrap_or_else(|error| unreachable!("{error:?}"));
    assert_eq!(
        outcome,
        WriteOutcome::Written(dir.path().join("trips/2026/beach.webp"))
    );
}

#[test]
fn unicode_and_spaces_in_names_work() {
    let dir = temp_dir();
    let outcome = write_ok(
        dir.path(),
        "Pâte à crêpes 🥞 'final' (2)",
        b"x",
        ConflictPolicy::Rename,
    );
    assert!(matches!(outcome, WriteOutcome::Written(path) if path.exists()));
}

#[test]
fn escaping_or_device_paths_are_refused() {
    let root = Path::new("/out");
    for bad in [
        "../escape.png",
        "a/../../b.png",
        "/etc/passwd",
        "",
        ".",
        "photos/CON.png",
        "nul",
        "lpt1.jpg",
        "com9",
    ] {
        assert!(
            matches!(
                destination(root, Path::new(bad)),
                Err(OutputError::UnsafePath)
            ),
            "{bad:?} should be refused"
        );
    }
    for good in [
        "a.png",
        "deep/nested/a.png",
        "CONTENT.png",
        "com10.png",
        "console.png",
    ] {
        assert!(
            destination(root, Path::new(good)).is_ok(),
            "{good:?} should be allowed"
        );
    }
}

#[test]
fn write_failure_reports_kind_and_leaves_nothing() {
    let dir = temp_dir();
    // A file where the output folder should be: creating the folder fails.
    let blocker = dir.path().join("blocked");
    std::fs::write(&blocker, b"").unwrap_or_else(|error| unreachable!("{error}"));
    let result = write_new(
        &blocker.join("sub"),
        OsStr::new("x"),
        "png",
        b"x",
        ConflictPolicy::Rename,
    );
    assert!(matches!(result, Err(OutputError::Io(_))), "{result:?}");
    assert!(leftovers(dir.path()).is_empty());
}

#[test]
fn names_differing_only_by_case_never_overwrite() {
    // On case-insensitive volumes (Windows, default macOS) "A" and "a" are
    // the same file, so the second write must be numbered. On case-sensitive
    // volumes both names are free. Either way the first file is untouched.
    let dir = temp_dir();
    let first = write_ok(dir.path(), "A", b"upper", ConflictPolicy::Rename);
    let second = write_ok(dir.path(), "a", b"lower", ConflictPolicy::Rename);
    let (WriteOutcome::Written(first), WriteOutcome::Written(second)) = (first, second) else {
        unreachable!("both writes should succeed");
    };
    assert_eq!(std::fs::read(&first).unwrap_or_default(), b"upper");
    assert_eq!(std::fs::read(&second).unwrap_or_default(), b"lower");
}

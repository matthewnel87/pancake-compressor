//! Scanner tests. Each test builds its own folder tree in a temporary folder.

use super::*;

fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"))
}

fn touch(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|error| unreachable!("{error}"));
    }
    fs::write(path, bytes).unwrap_or_else(|error| unreachable!("{error}"));
}

fn relatives(result: &ScanResult) -> Vec<String> {
    result
        .files
        .iter()
        .map(|f| f.relative.to_string_lossy().replace('\\', "/"))
        .collect()
}

fn reasons(result: &ScanResult) -> Vec<(String, &'static str)> {
    result
        .skipped
        .iter()
        .map(|s| {
            (
                s.relative.to_string_lossy().replace('\\', "/"),
                s.reason.code(),
            )
        })
        .collect()
}

#[test]
fn nested_files_keep_their_relative_paths_in_sorted_order() {
    let dir = temp_dir();
    touch(&dir.path().join("b.png"), b"1");
    touch(&dir.path().join("a/z.jpg"), b"22");
    touch(&dir.path().join("a/deeper/y.webp"), b"333");
    let result = scan(&[dir.path().to_path_buf()], None);
    assert_eq!(relatives(&result), ["a/deeper/y.webp", "a/z.jpg", "b.png"]);
    assert_eq!(result.files[1].size, 2);
    assert!(result.skipped.is_empty());
}

#[test]
fn output_folders_are_never_scanned() {
    let dir = temp_dir();
    touch(&dir.path().join("photo.png"), b"1");
    touch(&dir.path().join("_pancake/photo.webp"), b"1");
    touch(&dir.path().join("custom-out/photo.webp"), b"1");
    let out = dir.path().join("custom-out");
    let result = scan(&[dir.path().to_path_buf()], Some(&out));
    assert_eq!(relatives(&result), ["photo.png"]);
    let skipped: Vec<_> = reasons(&result).into_iter().map(|(p, _)| p).collect();
    assert_eq!(skipped, ["_pancake", "custom-out"]);
}

#[test]
fn overlapping_and_repeated_selections_are_listed_once() {
    let dir = temp_dir();
    touch(&dir.path().join("sub/a.png"), b"1");
    touch(&dir.path().join("b.png"), b"1");
    let roots = [
        dir.path().join("sub"),
        dir.path().to_path_buf(),
        dir.path().to_path_buf(),
        dir.path().join("b.png"),
    ];
    let result = scan(&roots, None);
    assert_eq!(relatives(&result), ["b.png", "sub/a.png"]);
    let codes: Vec<_> = reasons(&result).into_iter().map(|(_, c)| c).collect();
    assert_eq!(
        codes,
        ["already-included", "already-included", "already-included"]
    );
}

#[test]
fn os_clutter_is_ignored_silently() {
    let dir = temp_dir();
    for junk in [".DS_Store", "Thumbs.db", "desktop.ini", "._photo.png"] {
        touch(&dir.path().join(junk), b"x");
    }
    touch(&dir.path().join("photo.png"), b"x");
    let result = scan(&[dir.path().to_path_buf()], None);
    assert_eq!(relatives(&result), ["photo.png"]);
    assert!(result.skipped.is_empty());
}

#[test]
fn a_selected_single_file_uses_its_name_as_relative_path() {
    let dir = temp_dir();
    touch(&dir.path().join("one.png"), b"x");
    let result = scan(&[dir.path().join("one.png")], None);
    assert_eq!(relatives(&result), ["one.png"]);
}

#[test]
fn missing_roots_are_reported() {
    let dir = temp_dir();
    let missing = dir.path().join("gone");
    let result = scan(std::slice::from_ref(&missing), None);
    assert!(result.files.is_empty());
    assert_eq!(
        result.skipped,
        [Skipped {
            relative: missing,
            reason: SkipReason::Unreadable
        }]
    );
}

#[test]
fn very_deep_trees_stop_at_the_limit() {
    let dir = temp_dir();
    let mut deep = dir.path().to_path_buf();
    for _ in 0..(MAX_DEPTH + 3) {
        deep.push("d");
    }
    touch(&deep.join("lost.png"), b"x");
    touch(&dir.path().join("kept.png"), b"x");
    let result = scan(&[dir.path().to_path_buf()], None);
    assert_eq!(relatives(&result), ["kept.png"]);
    assert_eq!(
        reasons(&result)
            .iter()
            .filter(|(_, c)| *c == "folder-too-deep")
            .count(),
        1
    );
}

#[cfg(unix)]
mod links {
    use std::os::unix::fs::symlink;

    use super::*;

    #[test]
    fn symlinks_are_not_followed_even_into_loops_or_outside_folders() {
        let outside = temp_dir();
        touch(&outside.path().join("secret.png"), b"x");
        let dir = temp_dir();
        touch(&dir.path().join("real.png"), b"x");
        symlink(outside.path(), dir.path().join("to-outside"))
            .unwrap_or_else(|e| unreachable!("{e}"));
        symlink(dir.path(), dir.path().join("loop")).unwrap_or_else(|e| unreachable!("{e}"));
        symlink(
            outside.path().join("secret.png"),
            dir.path().join("file-link.png"),
        )
        .unwrap_or_else(|e| unreachable!("{e}"));

        let result = scan(&[dir.path().to_path_buf()], None);
        assert_eq!(relatives(&result), ["real.png"]);
        let mut skipped = reasons(&result);
        skipped.sort();
        assert_eq!(
            skipped,
            [
                ("file-link.png".to_owned(), "link-not-followed"),
                ("loop".to_owned(), "link-not-followed"),
                ("to-outside".to_owned(), "link-not-followed"),
            ]
        );
    }

    #[test]
    fn a_selected_root_that_is_a_link_is_refused() {
        let dir = temp_dir();
        touch(&dir.path().join("real/a.png"), b"x");
        let link = dir.path().join("link");
        symlink(dir.path().join("real"), &link).unwrap_or_else(|e| unreachable!("{e}"));
        let result = scan(std::slice::from_ref(&link), None);
        assert!(result.files.is_empty());
        assert_eq!(
            result.skipped,
            [Skipped {
                relative: link,
                reason: SkipReason::Link
            }]
        );
    }

    #[test]
    fn named_pipes_are_skipped() {
        let dir = temp_dir();
        let fifo = dir.path().join("pipe");
        let status = std::process::Command::new("mkfifo").arg(&fifo).status();
        if !status.is_ok_and(|s| s.success()) {
            return; // mkfifo unavailable; covered on other runners
        }
        let result = scan(&[dir.path().to_path_buf()], None);
        assert!(result.files.is_empty());
        assert_eq!(
            reasons(&result),
            [("pipe".to_owned(), "not-a-regular-file")]
        );
    }
}

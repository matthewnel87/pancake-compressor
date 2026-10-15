//! Batch engine acceptance tests (docs/17 M2). Every test builds its own
//! synthetic files in a temporary folder.

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::Mutex;

use image::{DynamicImage, ImageBuffer, Rgb};

use super::*;
use crate::scan::scan;

fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"))
}

/// A photo-like PNG that compresses well when re-encoded.
fn photo_png(seed: u32, size: u32) -> Vec<u8> {
    let mut state = seed | 1;
    let image = DynamicImage::ImageRgb8(ImageBuffer::from_fn(size, size, |x, y| {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let grain = (state % 16) as u8;
        Rgb([(x * 3) as u8 + grain, (y * 3) as u8 + grain, 90 + grain])
    }));
    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, image::ImageFormat::Png)
        .unwrap_or_else(|error| unreachable!("{error}"));
    out.into_inner()
}

fn write(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|error| unreachable!("{error}"));
    }
    std::fs::write(path, bytes).unwrap_or_else(|error| unreachable!("{error}"));
}

/// Runs a batch and records every event.
fn run_collect(
    files: &[ScannedFile],
    settings: &BatchSettings,
    cancel: &CancelToken,
    on_each: impl Fn(&Event) + Sync,
) -> (Summary, Vec<Event>) {
    let events = Mutex::new(Vec::new());
    let summary = run(files, settings, cancel, &|event: &Event| {
        on_each(event);
        if let Ok(mut list) = events.lock() {
            list.push(event.clone());
        }
    });
    (summary, events.into_inner().unwrap_or_default())
}

/// Checks the guarantees every batch must meet, whatever happened.
fn assert_reconciles(summary: &Summary, events: &[Event], total: usize) {
    assert_eq!(summary.total, total);
    assert_eq!(
        summary.completed
            + summary.not_smaller
            + summary.skipped
            + summary.failed
            + summary.cancelled,
        total,
        "{summary:?}"
    );
    // Strictly increasing sequence numbers, starting at 1.
    for (i, event) in events.iter().enumerate() {
        assert_eq!(event.sequence, i as u64 + 1);
        assert_eq!(event.total, total);
        assert_eq!(event.schema_version, EVENT_SCHEMA_VERSION);
    }
    // Exactly one final result per file; processed counts only those.
    let mut finished = HashMap::new();
    for event in events {
        if let Update::Finished(_) = event.update {
            *finished.entry(event.file_index).or_insert(0) += 1;
        }
    }
    assert_eq!(finished.len(), total);
    assert!(finished.values().all(|&n| n == 1));
    if total > 0 {
        assert_eq!(events.last().map(|e| e.processed), Some(total));
    }
}

fn settings_for(output: &Path) -> BatchSettings {
    let mut settings = BatchSettings::new(output.to_path_buf());
    settings.target = Target::WebP;
    settings
}

#[test]
fn mixed_batch_reports_every_file_and_writes_only_good_outputs() {
    let input = temp_dir();
    let output = temp_dir();
    write(&input.path().join("photos/a.png"), &photo_png(1, 64));
    write(&input.path().join("photos/b.png"), &photo_png(2, 64));
    write(&input.path().join("notes.txt"), b"not an image");
    write(
        &input.path().join("old.gif"),
        b"GIF89a\x01\x00\x01\x00\x00\x00\x00",
    );
    let mut broken = photo_png(3, 64);
    broken.truncate(broken.len() / 2);
    write(&input.path().join("broken.png"), &broken);
    let mut apng = photo_png(4, 16);
    // Insert an acTL chunk right after IHDR to make it an animated PNG.
    let actl = [
        0, 0, 0, 8, b'a', b'c', b'T', b'L', 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    apng.splice(33..33, actl);
    write(&input.path().join("moving.png"), &apng);

    let files = scan(&[input.path().to_path_buf()], Some(output.path())).files;
    let (summary, events) = run_collect(
        &files,
        &settings_for(output.path()),
        &CancelToken::new(),
        |_| {},
    );
    assert_reconciles(&summary, &events, 6);
    assert_eq!(summary.completed, 2, "{summary:?}");

    let results: HashMap<String, Finished> = events
        .iter()
        .filter_map(|e| match &e.update {
            Update::Finished(f) => Some((
                files[e.file_index]
                    .relative
                    .to_string_lossy()
                    .replace('\\', "/"),
                f.clone(),
            )),
            Update::Phase(_) => None,
        })
        .collect();
    assert_eq!(
        results["notes.txt"],
        Finished::Skipped {
            code: "unknown-format"
        }
    );
    assert_eq!(
        results["old.gif"],
        Finished::Skipped {
            code: "unsupported-format"
        }
    );
    assert_eq!(
        results["moving.png"],
        Finished::Skipped {
            code: "animated-image"
        }
    );
    assert_eq!(
        results["broken.png"],
        Finished::Failed {
            code: "corrupt-image-data"
        }
    );
    assert!(output.path().join("photos/a.webp").exists());
    assert!(output.path().join("photos/b.webp").exists());
}

#[test]
fn five_hundred_small_files_in_nested_folders_all_reconcile() {
    let input = temp_dir();
    let output = temp_dir();
    let png = photo_png(7, 24);
    for i in 0..500 {
        write(
            &input
                .path()
                .join(format!("set{}/sub{}/img{i}.png", i % 5, i % 3)),
            &png,
        );
    }
    let files = scan(&[input.path().to_path_buf()], Some(output.path())).files;
    assert_eq!(files.len(), 500);
    let (summary, events) = run_collect(
        &files,
        &settings_for(output.path()),
        &CancelToken::new(),
        |_| {},
    );
    assert_reconciles(&summary, &events, 500);
    assert_eq!(summary.failed, 0, "{summary:?}");
    // Every completed output exists at its mirrored relative path.
    for event in &events {
        if let Update::Finished(Finished::Completed { output: path, .. }) = &event.update {
            let relative = files[event.file_index].relative.with_extension("webp");
            assert_eq!(
                path,
                &output
                    .path()
                    .canonicalize()
                    .unwrap_or_default()
                    .join(relative)
            );
            assert!(path.exists());
        }
    }
}

#[test]
fn originals_are_never_modified() {
    let input = temp_dir();
    let output = temp_dir();
    let original = photo_png(9, 48);
    write(&input.path().join("keep.png"), &original);
    let files = scan(&[input.path().to_path_buf()], Some(output.path())).files;
    let mut settings = settings_for(output.path());
    settings.target = Target::Png; // same format, same name stem
    let _ = run(&files, &settings, &CancelToken::new(), &|_: &Event| {});
    assert_eq!(
        std::fs::read(input.path().join("keep.png")).unwrap_or_default(),
        original
    );
}

#[test]
fn existing_outputs_are_kept_and_new_ones_numbered() {
    let input = temp_dir();
    let output = temp_dir();
    write(&input.path().join("a.png"), &photo_png(11, 48));
    write(&output.path().join("a.webp"), b"user's existing file");
    let files = scan(&[input.path().to_path_buf()], Some(output.path())).files;
    let (summary, _) = run_collect(
        &files,
        &settings_for(output.path()),
        &CancelToken::new(),
        |_| {},
    );
    assert_eq!(summary.completed, 1);
    assert_eq!(
        std::fs::read(output.path().join("a.webp")).unwrap_or_default(),
        b"user's existing file"
    );
    assert!(output.path().join("a-pancake-2.webp").exists());

    let mut skip = settings_for(output.path());
    skip.conflict = ConflictPolicy::Skip;
    let (summary, events) = run_collect(&files, &skip, &CancelToken::new(), |_| {});
    assert_eq!(summary.skipped, 1);
    assert!(events.iter().any(|e| e.update
        == Update::Finished(Finished::Skipped {
            code: "output-exists"
        })));
}

#[test]
fn cancelling_before_start_writes_nothing() {
    let input = temp_dir();
    let output = temp_dir();
    for i in 0..10 {
        write(&input.path().join(format!("{i}.png")), &photo_png(i, 32));
    }
    let files = scan(&[input.path().to_path_buf()], Some(output.path())).files;
    let cancel = CancelToken::new();
    cancel.cancel();
    let (summary, events) = run_collect(&files, &settings_for(output.path()), &cancel, |_| {});
    assert_reconciles(&summary, &events, 10);
    assert_eq!(summary.cancelled, 10);
    let written = std::fs::read_dir(output.path())
        .map(|d| d.count())
        .unwrap_or(0);
    assert_eq!(written, 0);
}

#[test]
fn cancelling_mid_batch_keeps_finished_outputs_and_reconciles() {
    let input = temp_dir();
    let output = temp_dir();
    for i in 0..40 {
        write(
            &input.path().join(format!("{i:02}.png")),
            &photo_png(i + 100, 48),
        );
    }
    let files = scan(&[input.path().to_path_buf()], Some(output.path())).files;
    let cancel = CancelToken::new();
    let mut settings = settings_for(output.path());
    settings.workers = 2;
    let (summary, events) = run_collect(&files, &settings, &cancel, |event| {
        if matches!(event.update, Update::Finished(Finished::Completed { .. }))
            && event.processed >= 3
        {
            cancel.cancel();
        }
    });
    assert_reconciles(&summary, &events, 40);
    assert!(summary.completed >= 3, "{summary:?}");
    assert!(summary.cancelled > 0, "{summary:?}");
    // Every completed file is on disk and complete; no temporaries remain.
    for event in &events {
        if let Update::Finished(Finished::Completed {
            output: path,
            output_bytes,
            ..
        }) = &event.update
        {
            assert_eq!(
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
                *output_bytes
            );
        }
    }
    let temporaries = std::fs::read_dir(output.path())
        .map(|d| {
            d.filter_map(Result::ok)
                .filter(|e| e.file_name().to_string_lossy().starts_with(".pancake-"))
                .count()
        })
        .unwrap_or(1);
    assert_eq!(temporaries, 0);
}

#[test]
fn files_too_big_for_the_memory_budget_are_skipped_not_attempted() {
    let input = temp_dir();
    let output = temp_dir();
    write(&input.path().join("big.png"), &photo_png(5, 64));
    let files = scan(&[input.path().to_path_buf()], Some(output.path())).files;
    let mut settings = settings_for(output.path());
    settings.memory_budget = 1024; // far below any estimate
    let (summary, events) = run_collect(&files, &settings, &CancelToken::new(), |_| {});
    assert_eq!(summary.skipped, 1);
    assert!(events.iter().any(|e| e.update
        == Update::Finished(Finished::Skipped {
            code: "memory-budget-exceeded"
        })));
    // It never reached decoding.
    assert!(
        !events
            .iter()
            .any(|e| e.update == Update::Phase(Phase::Decoding))
    );
}

#[test]
fn concurrency_never_exceeds_the_worker_count() {
    let input = temp_dir();
    let output = temp_dir();
    for i in 0..24 {
        write(
            &input.path().join(format!("{i}.png")),
            &photo_png(i + 300, 40),
        );
    }
    let files = scan(&[input.path().to_path_buf()], Some(output.path())).files;
    let mut settings = settings_for(output.path());
    settings.workers = 3;
    let active = Mutex::new((HashSet::new(), 0usize)); // (in-flight files, peak)
    let (summary, _) = run_collect(&files, &settings, &CancelToken::new(), |event| {
        if let Ok(mut state) = active.lock() {
            match event.update {
                Update::Phase(Phase::Decoding) => {
                    state.0.insert(event.file_index);
                }
                Update::Finished(_) => {
                    state.0.remove(&event.file_index);
                }
                Update::Phase(_) => {}
            }
            state.1 = state.1.max(state.0.len());
        }
    });
    assert_eq!(summary.completed + summary.not_smaller, 24);
    let peak = active.lock().map(|s| s.1).unwrap_or(usize::MAX);
    assert!(peak <= 3, "peak concurrency {peak} exceeded 3 workers");
}

#[test]
fn an_unusable_output_folder_fails_every_file_cleanly() {
    let input = temp_dir();
    let blocker = temp_dir();
    write(&input.path().join("a.png"), &photo_png(1, 16));
    let not_a_folder = blocker.path().join("file");
    write(&not_a_folder, b"x");
    let files = scan(&[input.path().to_path_buf()], None).files;
    let (summary, events) = run_collect(
        &files,
        &settings_for(&not_a_folder),
        &CancelToken::new(),
        |_| {},
    );
    assert_reconciles(&summary, &events, 1);
    assert_eq!(summary.failed, 1);
}

#[test]
fn memory_estimate_is_monotonic_and_saturating() {
    assert!(estimate_memory(4000, 3000, 1) > estimate_memory(400, 300, 1));
    assert_eq!(estimate_memory(u32::MAX, u32::MAX, u64::MAX), u64::MAX);
}

#[test]
fn a_tiny_file_claiming_huge_dimensions_is_refused_before_decoding() {
    let input = temp_dir();
    let output = temp_dir();
    // Valid PNG signature + IHDR claiming 100,000 x 100,000 RGBA pixels
    // (40 GB once decoded) in about 60 bytes.
    let mut bomb = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13];
    bomb.extend(b"IHDR");
    bomb.extend(100_000u32.to_be_bytes());
    bomb.extend(100_000u32.to_be_bytes());
    bomb.extend([8, 6, 0, 0, 0, 0, 0, 0, 0]);
    bomb.extend([0, 0, 0, 0]);
    bomb.extend(b"IDAT");
    write(&input.path().join("bomb.png"), &bomb);
    let files = scan(&[input.path().to_path_buf()], Some(output.path())).files;
    let (summary, events) = run_collect(
        &files,
        &settings_for(output.path()),
        &CancelToken::new(),
        |_| {},
    );
    assert_eq!(summary.skipped, 1);
    assert!(events.iter().any(|e| e.update
        == Update::Finished(Finished::Skipped {
            code: "too-many-pixels"
        })));
    assert!(
        !events
            .iter()
            .any(|e| e.update == Update::Phase(Phase::Decoding))
    );
}

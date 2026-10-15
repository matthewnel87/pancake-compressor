//! Tests for the UI bridge logic that does not need a window.

use std::io::Cursor;

use image::{DynamicImage, ImageBuffer, Rgb};
use pancake_core::batch::Event;
use pancake_core::encode::Adjustments;
use pancake_core::format::ImageFormat;

use super::*;

fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| unreachable!("{error}"))
}

fn png() -> Vec<u8> {
    let image = DynamicImage::ImageRgb8(ImageBuffer::from_fn(8, 8, |x, y| {
        Rgb([x as u8 * 30, y as u8 * 30, 9])
    }));
    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, image::ImageFormat::Png)
        .unwrap_or_else(|error| unreachable!("{error}"));
    out.into_inner()
}

#[test]
fn registering_a_selection_counts_files_and_returns_an_opaque_token() {
    let dir = temp_dir();
    std::fs::create_dir_all(dir.path().join("Holiday")).unwrap_or_else(|e| unreachable!("{e}"));
    std::fs::write(dir.path().join("Holiday/a.png"), png()).unwrap_or_else(|e| unreachable!("{e}"));
    std::fs::write(dir.path().join("Holiday/b.png"), png()).unwrap_or_else(|e| unreachable!("{e}"));
    let state = AppState::default();
    let selection = register_selection(&state, vec![dir.path().join("Holiday")]);
    assert_eq!(selection.file_count, 2);
    assert_eq!(selection.label, "Holiday");
    assert!(selection.token.starts_with("in-"));
    // The token contains no part of the path.
    assert!(!selection.token.contains("Holiday"));
    let second = register_selection(&state, vec![dir.path().join("Holiday")]);
    assert_ne!(selection.token, second.token);
}

#[test]
fn default_output_is_a_pancake_folder_inside_or_beside_the_selection() {
    let dir = temp_dir();
    let file = dir.path().join("one.png");
    std::fs::write(&file, png()).unwrap_or_else(|e| unreachable!("{e}"));
    assert_eq!(
        default_output_for(&[dir.path().to_path_buf()]),
        Some(dir.path().join("_pancake"))
    );
    assert_eq!(
        default_output_for(&[file]),
        Some(dir.path().join("_pancake"))
    );
    assert_eq!(default_output_for(&[]), None);
}

#[test]
fn options_from_the_ui_are_validated_by_rust() {
    let good = r#"{"inputToken":"in-1","outputToken":null,"target":"auto","preset":"bestQuality","conflict":"skip"}"#;
    let parsed: BatchOptionsDto =
        serde_json::from_str(good).unwrap_or_else(|e| unreachable!("{e}"));
    assert_eq!(parsed.target, TargetDto::Auto);
    assert!(!parsed.allow_colour_profile_loss);
    let settings = settings_from(&parsed, PathBuf::from("/out"));
    assert_eq!(settings.target, Target::Auto);
    assert_eq!(settings.preset, Preset::BestQuality);
    assert_eq!(settings.conflict, ConflictPolicy::Skip);

    for bad in [
        r#"{"inputToken":"in-1","target":"gif","preset":"balanced","conflict":"rename"}"#,
        r#"{"inputToken":"in-1","target":"png","preset":"balanced","conflict":"overwrite"}"#,
        r#"{"inputToken":"in-1","target":"png","preset":"balanced","conflict":"rename","outputPath":"/etc"}"#,
    ] {
        assert!(
            serde_json::from_str::<BatchOptionsDto>(bad).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn events_carry_relative_names_and_notices_never_full_paths() {
    let event = Event {
        schema_version: 1,
        sequence: 7,
        file_index: 2,
        update: Update::Finished(Finished::Completed {
            output: PathBuf::from("/Users/someone/Pictures/_pancake/trip/a.jpg"),
            output_format: ImageFormat::Jpeg,
            original_bytes: 1000,
            output_bytes: 400,
            adjustments: Adjustments {
                transparency_flattened: true,
                ..Adjustments::default()
            },
            reason: "requested",
        }),
        processed: 3,
        total: 5,
    };
    let dto = event_dto("batch-1", &event, Path::new("trip/a.png"));
    assert_eq!(dto.state, "completed");
    assert_eq!(dto.file, "trip/a.png");
    assert_eq!(dto.notices, ["transparencyFlattened"]);
    assert_eq!(dto.output_format, Some("jpeg"));
    let json = serde_json::to_string(&dto).unwrap_or_default();
    assert!(!json.contains("/Users/"), "full path leaked: {json}");
    assert!(json.contains("\"schemaVersion\":1"));
}

#[test]
fn base64_matches_rfc_4648_vectors() {
    for (input, expected) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(base64(input.as_bytes()), expected, "{input:?}");
    }
    assert_eq!(base64(&[0xFB, 0xFF]), "+/8=");
}

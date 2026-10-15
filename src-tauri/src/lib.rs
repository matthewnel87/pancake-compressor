//! Desktop shell for Pancake Compressor.
//!
//! This crate is the only bridge between the web front end and the computer.
//! The front end is treated as untrusted: it can only call the commands
//! registered in [`run`], and each command validates its own input in Rust.
//!
//! Commands: [`commands::get_app_info`] (read-only) and the batch bridge in
//! [`batch`]: native pickers opened from Rust, token-based batch start,
//! cancellation and snapshots (`docs/03-architecture-and-ipc.md`). The web UI
//! never passes a file path that Rust trusts.

pub mod batch;
pub mod commands;

use tauri::Manager;

/// Builds and runs the Tauri application.
///
/// Only the commands listed in `generate_handler!` are callable from the
/// front end, and `capabilities/main.json` limits which built-in Tauri APIs
/// the window may use.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        // Used from Rust only: the webview gets no dialog permission.
        .plugin(tauri_plugin_dialog::init())
        .manage(batch::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            batch::select_inputs,
            batch::select_output_folder,
            batch::start_batch,
            batch::cancel_batch,
            batch::get_batch_snapshot,
            batch::open_output_folder,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::DragDrop(drop) = event {
                batch::handle_drag_drop(window.app_handle(), drop);
            }
        })
        .run(tauri::generate_context!());

    if let Err(error) = result {
        // There is no window to show an error in at this point, so stderr is
        // the only channel. The message contains no user paths or content.
        eprintln!("Pancake Compressor failed to start: {error}");
        std::process::exit(1);
    }
}

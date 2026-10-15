//! Desktop shell for Pancake Compressor.
//!
//! This crate is the only bridge between the web front end and the computer.
//! The front end is treated as untrusted: it can only call the commands
//! registered in [`run`], and each command validates its own input in Rust.
//!
//! As of M0 there is a single read-only command, [`commands::get_app_info`],
//! which needs no file or network permission. File registration, batches and
//! cancellation are added in M2 following `docs/03-architecture-and-ipc.md`.

pub mod commands;

/// Builds and runs the Tauri application.
///
/// Only the commands listed in `generate_handler!` are callable from the
/// front end, and `capabilities/main.json` limits which built-in Tauri APIs
/// the window may use.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::get_app_info])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        // There is no window to show an error in at this point, so stderr is
        // the only channel. The message contains no user paths or content.
        eprintln!("Pancake Compressor failed to start: {error}");
        std::process::exit(1);
    }
}

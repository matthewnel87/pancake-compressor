//! Pancake Compressor's image engine.
//!
//! This crate owns everything that touches image bytes: format sniffing,
//! safety limits, decoding, encoding and safe output. It deliberately has no
//! knowledge of Tauri, windows or the web front end, so it can be tested on
//! its own and reused by a future command-line tool.
//!
//! **What it does not own:** user permission to read or write a path. The
//! desktop shell (`src-tauri`) registers paths chosen by the user and only
//! then hands them to this crate.
//!
//! **Invariants every module must keep** (see `AGENTS.md`):
//! - no network access of any kind;
//! - no panics on untrusted input (`unwrap`, `expect` and `panic!` are denied
//!   by lint in this crate);
//! - no `unsafe` code (forbidden by lint; revisit only with an ADR).
//!
//! As of M0 the crate only exposes build information and the memory
//! arithmetic that later decoding work depends on. Codec support arrives in
//! M1 and is tracked in `docs/IMPLEMENTATION-STATUS.md`.

pub mod limits;

/// Version of the engine crate, taken from Cargo at compile time.
///
/// The desktop app reports this alongside its own version so bug reports can
/// say exactly which engine produced an output file.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Whether this build can make network requests at run time.
///
/// Always `false`. It exists as a named constant so the privacy promise shown
/// in the app is tied to code. The compile-time assertion below makes the
/// build fail if anyone flips it without also revisiting `PRIVACY.md` and
/// ADR-004.
pub const RUNTIME_NETWORK_ACCESS: bool = false;
const _: () = assert!(
    !RUNTIME_NETWORK_ACCESS,
    "ADR-004: no runtime network access"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_version_matches_workspace_version() {
        assert_eq!(ENGINE_VERSION, "0.1.0");
    }
}

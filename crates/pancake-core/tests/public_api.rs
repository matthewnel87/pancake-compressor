//! Integration test: the engine's public surface works from outside the crate,
//! the way the desktop shell and a future CLI will use it.

use pancake_core::{ENGINE_VERSION, RUNTIME_NETWORK_ACCESS, limits};

#[test]
fn public_constants_are_reachable() {
    assert!(!ENGINE_VERSION.is_empty());
    // Read through a function boundary so this exercises the exported value.
    assert!(!std::hint::black_box(RUNTIME_NETWORK_ACCESS));
}

#[test]
fn hostile_dimensions_are_refused_through_public_api() {
    let one_gib = 1024 * 1024 * 1024;
    // 40,000 x 40,000 RGBA needs 6.4 GB: must not fit a 1 GiB budget.
    assert!(!limits::fits_memory_budget(40_000, 40_000, one_gib));
    assert!(limits::fits_memory_budget(1920, 1080, one_gib));
}

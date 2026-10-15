//! Commands the front end may call over Tauri IPC.
//!
//! Every command here is part of a public contract with `src/lib/ipc.ts`.
//! Field names are serialised in camelCase to match TypeScript conventions;
//! change both sides together and keep the `schema_version` in step.

use serde::Serialize;

/// Build and privacy facts shown on the launch screen and in bug reports.
///
/// Contains no user data and no file paths, so it is safe to display and to
/// copy into a public issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// Bumped whenever a field is added, removed or changes meaning.
    pub schema_version: u32,
    /// Desktop app version from Cargo (matches `tauri.conf.json`).
    pub app_version: &'static str,
    /// Version of the `pancake-core` engine linked into this build.
    pub engine_version: &'static str,
    /// Always `false`: the installed app makes no network requests (ADR-004).
    pub network_access: bool,
    /// Always `false`: no analytics, crash upload or usage tracking.
    pub telemetry: bool,
}

/// Returns version and privacy information for the running build.
///
/// Read-only and infallible, so it needs no capability beyond the default
/// command allowance.
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        schema_version: 1,
        app_version: env!("CARGO_PKG_VERSION"),
        engine_version: pancake_core::ENGINE_VERSION,
        network_access: pancake_core::RUNTIME_NETWORK_ACCESS,
        telemetry: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_reports_offline_and_no_telemetry() {
        let info = get_app_info();
        assert!(!info.network_access);
        assert!(!info.telemetry);
        assert_eq!(info.app_version, info.engine_version);
    }

    #[test]
    fn app_info_serialises_with_camel_case_contract() {
        let json = serde_json::to_value(get_app_info()).unwrap_or_default();
        for key in [
            "schemaVersion",
            "appVersion",
            "engineVersion",
            "networkAccess",
            "telemetry",
        ] {
            assert!(json.get(key).is_some(), "missing IPC field {key}");
        }
    }
}

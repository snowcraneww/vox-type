use crate::overlay::{dictation_overlay_label, overlay_size, overlay_url, OverlayBackendResult};

#[test]
fn overlay_uses_stable_label_and_route() {
    assert_eq!(dictation_overlay_label(), "dictation-overlay");
    assert_eq!(overlay_url(), "index.html?view=overlay");
}

#[test]
fn overlay_size_matches_tauri_window_config() {
    assert_eq!(overlay_size(), (120, 36));
}

#[test]
fn overlay_window_disables_shadow_in_tauri_config() {
    let config_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let config = std::fs::read_to_string(config_path).expect("tauri config should exist");
    let json: serde_json::Value =
        serde_json::from_str(&config).expect("tauri config should be json");
    let windows = json["app"]["windows"]
        .as_array()
        .expect("windows should be an array");
    let overlay = windows
        .iter()
        .find(|window| window["label"] == "dictation-overlay")
        .expect("dictation-overlay window should exist");

    assert_eq!(overlay["transparent"], true);
    assert_eq!(overlay["decorations"], false);
    assert_eq!(overlay["shadow"], false);
    assert_eq!(overlay["width"], 120);
    assert_eq!(overlay["height"], 36);
}

#[test]
fn overlay_backend_status_reports_native_windows_preference() {
    let status = crate::overlay::backend_status();

    assert_eq!(status.backend, "native-win32");
    assert_eq!(status.last_error, None);
}

#[test]
fn overlay_backend_result_reports_native_success() {
    let status = crate::overlay::resolve_overlay_backend(OverlayBackendResult::Native);

    assert_eq!(status.backend, "native-win32");
    assert_eq!(status.last_error, None);
}

#[test]
fn transcribing_overlay_command_is_available() {
    let command: fn(&tauri::AppHandle) -> Result<(), crate::error::VoxError> =
        crate::overlay::show_transcribing_overlay;

    assert!(!std::ptr::fn_addr_eq(
        command,
        crate::overlay::show_dictation_overlay
            as fn(&tauri::AppHandle) -> Result<(), crate::error::VoxError>
    ));
}

#[test]
fn overlay_backend_result_reports_webview_fallback_reason() {
    let status = crate::overlay::resolve_overlay_backend(OverlayBackendResult::FallbackWebview {
        error: "native overlay failed".to_string(),
    });

    assert_eq!(status.backend, "fallback-webview");
    assert_eq!(status.last_error, Some("native overlay failed".to_string()));
}

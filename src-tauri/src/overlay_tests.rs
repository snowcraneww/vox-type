use crate::overlay::{dictation_overlay_label, overlay_size, overlay_url};

#[test]
fn overlay_uses_stable_label_and_route() {
    assert_eq!(dictation_overlay_label(), "dictation-overlay");
    assert_eq!(overlay_url(), "index.html?view=overlay");
}

#[test]
fn overlay_size_matches_tauri_window_config() {
    assert_eq!(overlay_size(), (120, 32));
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
}

#[test]
fn overlay_backend_status_reports_shadowless_webview() {
    let status = crate::overlay::backend_status();

    assert_eq!(status.backend, "webview-shadowless");
    assert_eq!(status.last_error, None);
}

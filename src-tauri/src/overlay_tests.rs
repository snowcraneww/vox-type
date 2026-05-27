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

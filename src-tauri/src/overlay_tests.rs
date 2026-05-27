use crate::overlay::{dictation_overlay_label, overlay_url};

#[test]
fn overlay_uses_stable_label_and_route() {
    assert_eq!(dictation_overlay_label(), "dictation-overlay");
    assert_eq!(overlay_url(), "index.html?view=overlay");
}

use crate::error::VoxError;
use serde::Serialize;
use tauri::{AppHandle, Manager, PhysicalPosition};

pub const DICTATION_OVERLAY_LABEL: &str = "dictation-overlay";
pub const DICTATION_OVERLAY_URL: &str = "index.html?view=overlay";
const OVERLAY_WIDTH: i32 = 120;
const OVERLAY_HEIGHT: i32 = 32;
const OVERLAY_MARGIN_BOTTOM: i32 = 92;

pub fn dictation_overlay_label() -> &'static str {
    DICTATION_OVERLAY_LABEL
}

pub fn overlay_url() -> &'static str {
    DICTATION_OVERLAY_URL
}

pub fn overlay_size() -> (i32, i32) {
    (OVERLAY_WIDTH, OVERLAY_HEIGHT)
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OverlayBackendStatus {
    pub backend: String,
    pub last_error: Option<String>,
}

pub fn backend_status() -> OverlayBackendStatus {
    OverlayBackendStatus {
        backend: "webview-shadowless".to_string(),
        last_error: None,
    }
}

pub fn show_dictation_overlay(app: &AppHandle) -> Result<(), VoxError> {
    let window = app
        .get_webview_window(DICTATION_OVERLAY_LABEL)
        .ok_or_else(|| {
            VoxError::Config(format!("找不到桌面浮窗窗口：{DICTATION_OVERLAY_LABEL}"))
        })?;

    if let Ok(Some(monitor)) = window.primary_monitor() {
        let work_area = monitor.work_area();
        let x = work_area.position.x + ((work_area.size.width as i32 - OVERLAY_WIDTH) / 2);
        let y = work_area.position.y + work_area.size.height as i32
            - OVERLAY_HEIGHT
            - OVERLAY_MARGIN_BOTTOM;
        window
            .set_position(PhysicalPosition::new(x, y))
            .map_err(|error| VoxError::Config(format!("定位桌面浮窗失败：{error}")))?;
    }

    window
        .show()
        .map_err(|error| VoxError::Config(format!("显示桌面浮窗失败：{error}")))?;
    window
        .set_always_on_top(true)
        .map_err(|error| VoxError::Config(format!("设置桌面浮窗置顶失败：{error}")))?;
    Ok(())
}

pub fn hide_dictation_overlay(app: &AppHandle) -> Result<(), VoxError> {
    let window = app
        .get_webview_window(DICTATION_OVERLAY_LABEL)
        .ok_or_else(|| {
            VoxError::Config(format!("找不到桌面浮窗窗口：{DICTATION_OVERLAY_LABEL}"))
        })?;
    window
        .hide()
        .map_err(|error| VoxError::Config(format!("隐藏桌面浮窗失败：{error}")))
}

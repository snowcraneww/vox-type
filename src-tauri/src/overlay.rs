use crate::error::VoxError;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager, PhysicalPosition};

pub const DICTATION_OVERLAY_LABEL: &str = "dictation-overlay";
pub const DICTATION_OVERLAY_URL: &str = "index.html?view=overlay";
const OVERLAY_WIDTH: i32 = 132;
const OVERLAY_HEIGHT: i32 = 44;
const OVERLAY_MARGIN_BOTTOM: i32 = 92;

static BACKEND_STATUS: OnceLock<Mutex<OverlayBackendStatus>> = OnceLock::new();

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

pub enum OverlayBackendResult {
    Native,
    FallbackWebview { error: String },
}

pub fn resolve_overlay_backend(result: OverlayBackendResult) -> OverlayBackendStatus {
    match result {
        OverlayBackendResult::Native => OverlayBackendStatus {
            backend: "native-win32".to_string(),
            last_error: None,
        },
        OverlayBackendResult::FallbackWebview { error } => OverlayBackendStatus {
            backend: "fallback-webview".to_string(),
            last_error: Some(error),
        },
    }
}

pub fn backend_status() -> OverlayBackendStatus {
    current_backend_status()
}

pub fn show_dictation_overlay(app: &AppHandle) -> Result<(), VoxError> {
    show_overlay_with_native_result(app, crate::native_overlay::show_native_overlay())
}

pub fn show_transcribing_overlay(app: &AppHandle) -> Result<(), VoxError> {
    show_overlay_with_native_result(
        app,
        crate::native_overlay::show_native_transcribing_overlay(),
    )
}

fn show_overlay_with_native_result(
    app: &AppHandle,
    native_result: Result<(), VoxError>,
) -> Result<(), VoxError> {
    match native_result {
        Ok(()) => {
            set_backend_status(resolve_overlay_backend(OverlayBackendResult::Native));
            if let Some(window) = app.get_webview_window(DICTATION_OVERLAY_LABEL) {
                let _ = window.hide();
            }
            return Ok(());
        }
        Err(error) => {
            set_backend_status(resolve_overlay_backend(
                OverlayBackendResult::FallbackWebview {
                    error: error.to_string(),
                },
            ));
        }
    }

    show_webview_overlay(app)
}

pub fn hide_dictation_overlay(app: &AppHandle) -> Result<(), VoxError> {
    let native_result = crate::native_overlay::hide_native_overlay();

    if backend_status().backend == "native-win32" {
        return native_result;
    }

    hide_webview_overlay(app)
}

fn show_webview_overlay(app: &AppHandle) -> Result<(), VoxError> {
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

fn hide_webview_overlay(app: &AppHandle) -> Result<(), VoxError> {
    let window = app
        .get_webview_window(DICTATION_OVERLAY_LABEL)
        .ok_or_else(|| {
            VoxError::Config(format!("找不到桌面浮窗窗口：{DICTATION_OVERLAY_LABEL}"))
        })?;
    window
        .hide()
        .map_err(|error| VoxError::Config(format!("隐藏桌面浮窗失败：{error}")))
}

fn current_backend_status() -> OverlayBackendStatus {
    status_cell()
        .lock()
        .map(|status| status.clone())
        .unwrap_or_else(|error| OverlayBackendStatus {
            backend: "fallback-webview".to_string(),
            last_error: Some(format!("读取浮窗后端状态失败：{error}")),
        })
}

fn set_backend_status(status: OverlayBackendStatus) {
    if let Ok(mut current_status) = status_cell().lock() {
        *current_status = status;
    }
}

fn status_cell() -> &'static Mutex<OverlayBackendStatus> {
    BACKEND_STATUS.get_or_init(|| {
        Mutex::new(OverlayBackendStatus {
            backend: preferred_backend_name().to_string(),
            last_error: None,
        })
    })
}

fn preferred_backend_name() -> &'static str {
    #[cfg(windows)]
    {
        "native-win32"
    }

    #[cfg(not(windows))]
    {
        "webview-shadowless"
    }
}

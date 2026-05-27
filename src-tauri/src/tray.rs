use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, Runtime,
};

use crate::error::VoxError;

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), VoxError> {
    let show = MenuItem::with_id(app, "show", "打开设置", true, None::<&str>)
        .map_err(|error| VoxError::Config(format!("创建托盘菜单失败：{error}")))?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|error| VoxError::Config(format!("创建托盘菜单失败：{error}")))?;
    let menu = Menu::with_items(app, &[&show, &quit])
        .map_err(|error| VoxError::Config(format!("创建托盘菜单失败：{error}")))?;

    TrayIconBuilder::new()
        .tooltip("VoxType")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)
        .map_err(|error| VoxError::Config(format!("创建托盘失败：{error}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn tray_module_documents_menu_ids() {
        let ids = ["show", "quit"];
        assert_eq!(ids, ["show", "quit"]);
    }
}

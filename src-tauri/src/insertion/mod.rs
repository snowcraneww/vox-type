use std::time::Duration;

use crate::error::VoxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardPlan {
    pub text: String,
    pub paste_shortcut: &'static str,
    pub restore_clipboard: bool,
    pub verify_clipboard: bool,
    pub before_paste_delay_ms: u64,
    pub before_restore_delay_ms: u64,
}

pub trait InsertionStrategy {
    fn insert_text(&self, text: &str) -> Result<(), VoxError>;
}

#[derive(Debug, Default)]
pub struct MockInsertion;

impl InsertionStrategy for MockInsertion {
    fn insert_text(&self, text: &str) -> Result<(), VoxError> {
        if text.trim().is_empty() {
            return Err(VoxError::Insertion("不能上屏空文本".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ClipboardInsertion;

impl ClipboardInsertion {
    pub fn plan(text: &str) -> Result<ClipboardPlan, VoxError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(VoxError::Insertion("不能上屏空文本".to_string()));
        }
        Ok(ClipboardPlan {
            text: trimmed.to_string(),
            paste_shortcut: "Ctrl+V",
            restore_clipboard: false,
            verify_clipboard: true,
            before_paste_delay_ms: 120,
            before_restore_delay_ms: 800,
        })
    }
}

impl InsertionStrategy for ClipboardInsertion {
    fn insert_text(&self, text: &str) -> Result<(), VoxError> {
        let plan = Self::plan(text)?;
        insert_with_clipboard(&plan)
    }
}

#[cfg(windows)]
fn insert_with_clipboard(plan: &ClipboardPlan) -> Result<(), VoxError> {
    use arboard::Clipboard;
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    let mut clipboard = Clipboard::new()
        .map_err(|error| VoxError::Insertion(format!("打开剪贴板失败：{error}")))?;
    let previous = clipboard.get_text().ok();
    clipboard
        .set_text(plan.text.clone())
        .map_err(|error| VoxError::Insertion(format!("写入剪贴板失败：{error}")))?;
    if plan.verify_clipboard {
        let current = clipboard
            .get_text()
            .map_err(|error| VoxError::Insertion(format!("验证剪贴板写入失败：{error}")))?;
        if current != plan.text {
            return Err(VoxError::Insertion(
                "剪贴板写入后读回的文本不一致".to_string(),
            ));
        }
    }
    std::thread::sleep(Duration::from_millis(plan.before_paste_delay_ms));

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|error| VoxError::Insertion(format!("初始化键盘模拟失败：{error}")))?;
    enigo
        .key(Key::Control, Direction::Press)
        .and_then(|_| enigo.key(Key::V, Direction::Click))
        .and_then(|_| enigo.key(Key::Control, Direction::Release))
        .map_err(|error| VoxError::Insertion(format!("发送 Ctrl+V 失败：{error}")))?;
    std::thread::sleep(Duration::from_millis(plan.before_restore_delay_ms));

    if plan.restore_clipboard {
        if let Some(previous) = previous {
            clipboard
                .set_text(previous)
                .map_err(|error| VoxError::Insertion(format!("恢复剪贴板失败：{error}")))?;
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn insert_with_clipboard(_plan: &ClipboardPlan) -> Result<(), VoxError> {
    Err(VoxError::Insertion(
        "剪贴板上屏 MVP 目前只支持 Windows".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_insertion_rejects_empty_text() {
        let error = MockInsertion.insert_text(" ").unwrap_err();
        assert!(error.to_string().contains("不能上屏空文本"));
    }

    #[test]
    fn clipboard_plan_rejects_empty_text() {
        let error = ClipboardInsertion::plan(" ").unwrap_err();
        assert!(error.to_string().contains("不能上屏空文本"));
    }

    #[test]
    fn clipboard_plan_uses_ctrl_v_without_restoring_old_clipboard() {
        let plan = ClipboardInsertion::plan("你好 VoxType").unwrap();
        assert_eq!(plan.text, "你好 VoxType");
        assert_eq!(plan.paste_shortcut, "Ctrl+V");
        assert!(!plan.restore_clipboard);
        assert!(plan.verify_clipboard);
        assert!(plan.before_paste_delay_ms > 0);
        assert!(plan.before_restore_delay_ms >= 500);
    }
}

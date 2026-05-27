use crate::error::VoxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardPlan {
    pub text: String,
    pub paste_shortcut: &'static str,
    pub restore_clipboard: bool,
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
            restore_clipboard: true,
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

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|error| VoxError::Insertion(format!("初始化键盘模拟失败：{error}")))?;
    enigo
        .key(Key::Control, Direction::Press)
        .and_then(|_| enigo.key(Key::Unicode('v'), Direction::Click))
        .and_then(|_| enigo.key(Key::Control, Direction::Release))
        .map_err(|error| VoxError::Insertion(format!("发送 Ctrl+V 失败：{error}")))?;

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
    fn clipboard_plan_uses_ctrl_v_and_restore() {
        let plan = ClipboardInsertion::plan("你好 VoxType").unwrap();
        assert_eq!(plan.text, "你好 VoxType");
        assert_eq!(plan.paste_shortcut, "Ctrl+V");
        assert!(plan.restore_clipboard);
    }
}

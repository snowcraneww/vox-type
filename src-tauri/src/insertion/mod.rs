use std::time::Duration;

use crate::error::VoxError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InsertionStrategyId {
    Clipboard,
    SendInput,
    Auto,
}

impl Default for InsertionStrategyId {
    fn default() -> Self {
        Self::Clipboard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InsertionResult {
    pub requested_strategy: InsertionStrategyId,
    pub actual_strategy: InsertionStrategyId,
    pub fallback_used: bool,
    pub error_category: Option<String>,
}

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

#[derive(Debug, Default)]
pub struct SendInputInsertion;

impl InsertionStrategy for SendInputInsertion {
    fn insert_text(&self, text: &str) -> Result<(), VoxError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(VoxError::Insertion("empty insertion text".to_string()));
        }
        insert_with_sendinput(trimmed)
    }
}

pub fn insert_text_with_strategy(
    text: &str,
    strategy: InsertionStrategyId,
) -> Result<InsertionResult, VoxError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(VoxError::Insertion("empty insertion text".to_string()));
    }

    match strategy {
        InsertionStrategyId::Clipboard => {
            ClipboardInsertion.insert_text(trimmed)?;
            Ok(insertion_result(
                InsertionStrategyId::Clipboard,
                InsertionStrategyId::Clipboard,
            ))
        }
        InsertionStrategyId::SendInput => {
            SendInputInsertion.insert_text(trimmed)?;
            Ok(insertion_result(
                InsertionStrategyId::SendInput,
                InsertionStrategyId::SendInput,
            ))
        }
        InsertionStrategyId::Auto => match SendInputInsertion.insert_text(trimmed) {
            Ok(()) => Ok(insertion_result(
                InsertionStrategyId::Auto,
                InsertionStrategyId::SendInput,
            )),
            Err(sendinput_error) => {
                ClipboardInsertion.insert_text(trimmed)?;
                Ok(insertion_result_for_fallback(
                    InsertionStrategyId::Auto,
                    InsertionStrategyId::Clipboard,
                    Some(categorize_insertion_error(&sendinput_error)),
                ))
            }
        },
    }
}

fn insertion_result(
    requested_strategy: InsertionStrategyId,
    actual_strategy: InsertionStrategyId,
) -> InsertionResult {
    InsertionResult {
        requested_strategy,
        actual_strategy,
        fallback_used: false,
        error_category: None,
    }
}

pub fn insertion_result_for_fallback(
    requested_strategy: InsertionStrategyId,
    actual_strategy: InsertionStrategyId,
    error_category: Option<String>,
) -> InsertionResult {
    InsertionResult {
        requested_strategy,
        actual_strategy,
        fallback_used: true,
        error_category,
    }
}

fn categorize_insertion_error(error: &VoxError) -> String {
    let message = error.to_string().to_ascii_lowercase();
    if message.contains("sendinput") {
        return "sendinput_failed".to_string();
    }
    if message.contains("unsupported") || message.contains("windows") {
        return "unsupported_platform".to_string();
    }
    "insertion_failed".to_string()
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

#[cfg(windows)]
#[derive(Clone, Copy)]
#[repr(C)]
struct KeyboardInput {
    w_vk: u16,
    w_scan: u16,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[cfg(windows)]
#[derive(Clone, Copy)]
#[repr(C)]
union InputUnion {
    ki: KeyboardInput,
}

#[cfg(windows)]
#[derive(Clone, Copy)]
#[repr(C)]
struct Input {
    input_type: u32,
    union: InputUnion,
}

#[cfg(windows)]
extern "system" {
    fn SendInput(c_inputs: u32, p_inputs: *const Input, cb_size: i32) -> u32;
}

#[cfg(windows)]
fn insert_with_sendinput(text: &str) -> Result<(), VoxError> {
    const INPUT_KEYBOARD: u32 = 1;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const KEYEVENTF_UNICODE: u32 = 0x0004;

    let mut inputs = Vec::with_capacity(text.encode_utf16().count() * 2);
    for code_unit in text.encode_utf16() {
        inputs.push(Input {
            input_type: INPUT_KEYBOARD,
            union: InputUnion {
                ki: KeyboardInput {
                    w_vk: 0,
                    w_scan: code_unit,
                    dw_flags: KEYEVENTF_UNICODE,
                    time: 0,
                    dw_extra_info: 0,
                },
            },
        });
        inputs.push(Input {
            input_type: INPUT_KEYBOARD,
            union: InputUnion {
                ki: KeyboardInput {
                    w_vk: 0,
                    w_scan: code_unit,
                    dw_flags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dw_extra_info: 0,
                },
            },
        });
    }

    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<Input>() as i32,
        )
    };
    if sent != inputs.len() as u32 {
        return Err(VoxError::Insertion(format!(
            "SendInput failed: {sent}/{}",
            inputs.len()
        )));
    }
    Ok(())
}

#[cfg(not(windows))]
fn insert_with_sendinput(_text: &str) -> Result<(), VoxError> {
    Err(VoxError::Insertion(
        "SendInput Unicode insertion only supports Windows".to_string(),
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

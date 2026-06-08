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
        InsertionStrategyId::Auto => {
            ClipboardInsertion.insert_text(trimmed)?;
            Ok(insertion_result_for_fallback(
                InsertionStrategyId::Auto,
                InsertionStrategyId::Clipboard,
                Some("auto_clipboard_policy".to_string()),
            ))
        }
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
#[allow(dead_code)]
struct MouseInput {
    dx: i32,
    dy: i32,
    mouse_data: u32,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[cfg(windows)]
#[derive(Clone, Copy)]
#[repr(C)]
union InputUnion {
    ki: KeyboardInput,
    mi: MouseInput,
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
    fn VkKeyScanW(ch: u16) -> i16;
    fn GetAsyncKeyState(v_key: i32) -> i16;
}

#[cfg(windows)]
const INPUT_KEYBOARD: u32 = 1;
#[cfg(windows)]
const KEYEVENTF_KEYUP: u32 = 0x0002;
#[cfg(windows)]
const VK_SHIFT: u16 = 0x10;
#[cfg(windows)]
const VK_CONTROL: u16 = 0x11;
#[cfg(windows)]
const VK_MENU: u16 = 0x12;
#[cfg(windows)]
const VK_LWIN: u16 = 0x5b;
#[cfg(windows)]
const VK_RWIN: u16 = 0x5c;
#[cfg(windows)]
const MODIFIER_KEYS: [u16; 5] = [VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN];

#[cfg(windows)]
fn keyboard_input(vk: u16, flags: u32) -> Input {
    Input {
        input_type: INPUT_KEYBOARD,
        union: InputUnion {
            ki: KeyboardInput {
                w_vk: vk,
                w_scan: 0,
                dw_flags: flags,
                time: 0,
                dw_extra_info: 0,
            },
        },
    }
}

#[cfg(windows)]
fn modifier_keyup_inputs() -> Vec<Input> {
    MODIFIER_KEYS
        .iter()
        .map(|key| keyboard_input(*key, KEYEVENTF_KEYUP))
        .collect()
}

#[cfg(windows)]
fn is_modifier_down() -> bool {
    MODIFIER_KEYS.iter().any(|key| {
        let state = unsafe { GetAsyncKeyState(i32::from(*key)) };
        state < 0
    })
}

#[cfg(windows)]
fn wait_for_physical_modifiers_to_release() {
    let deadline = std::time::Instant::now() + Duration::from_millis(400);
    while is_modifier_down() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(windows)]
fn clear_modifier_state() -> Result<(), VoxError> {
    wait_for_physical_modifiers_to_release();
    send_keyboard_inputs(&modifier_keyup_inputs())
}

#[cfg(windows)]
fn ascii_key_input_sequence(character: char) -> Result<Vec<Input>, VoxError> {
    if !character.is_ascii() {
        return Err(VoxError::Insertion(
            "SendInput virtual-key insertion only supports ASCII text".to_string(),
        ));
    }

    let scan = unsafe { VkKeyScanW(character as u16) };
    if scan == -1 {
        return Err(VoxError::Insertion(format!(
            "SendInput virtual-key mapping failed for {character:?}"
        )));
    }

    let vk = (scan as u16) & 0x00ff;
    let shift_state = ((scan as u16) >> 8) & 0x00ff;
    let mut inputs = Vec::with_capacity(8);

    if shift_state & 0x01 != 0 {
        inputs.push(keyboard_input(VK_SHIFT, 0));
    }
    if shift_state & 0x02 != 0 {
        inputs.push(keyboard_input(VK_CONTROL, 0));
    }
    if shift_state & 0x04 != 0 {
        inputs.push(keyboard_input(VK_MENU, 0));
    }

    inputs.push(keyboard_input(vk, 0));
    inputs.push(keyboard_input(vk, KEYEVENTF_KEYUP));

    if shift_state & 0x04 != 0 {
        inputs.push(keyboard_input(VK_MENU, KEYEVENTF_KEYUP));
    }
    if shift_state & 0x02 != 0 {
        inputs.push(keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP));
    }
    if shift_state & 0x01 != 0 {
        inputs.push(keyboard_input(VK_SHIFT, KEYEVENTF_KEYUP));
    }

    Ok(inputs)
}

#[cfg(windows)]
fn send_keyboard_inputs(inputs: &[Input]) -> Result<(), VoxError> {
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

#[cfg(windows)]
fn insert_with_sendinput(text: &str) -> Result<(), VoxError> {
    clear_modifier_state()?;
    for character in text.chars() {
        let inputs = ascii_key_input_sequence(character)?;
        send_keyboard_inputs(&inputs)?;
        std::thread::sleep(Duration::from_millis(2));
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

    #[test]
    fn auto_strategy_reports_clipboard_policy_metadata() {
        let result = insertion_result_for_fallback(
            InsertionStrategyId::Auto,
            InsertionStrategyId::Clipboard,
            Some("auto_clipboard_policy".to_string()),
        );

        assert_eq!(result.requested_strategy, InsertionStrategyId::Auto);
        assert_eq!(result.actual_strategy, InsertionStrategyId::Clipboard);
        assert!(result.fallback_used);
        assert_eq!(
            result.error_category.as_deref(),
            Some("auto_clipboard_policy")
        );
    }

    #[cfg(all(test, windows))]
    #[test]
    fn modifier_keyup_inputs_release_common_shortcut_modifiers() {
        let inputs = modifier_keyup_inputs();

        assert_eq!(inputs.len(), MODIFIER_KEYS.len());
        for (input, expected_key) in inputs.iter().zip(MODIFIER_KEYS.iter()) {
            assert_eq!(input.input_type, INPUT_KEYBOARD);
            let keyup = unsafe { input.union.ki };
            assert_eq!(keyup.w_vk, *expected_key);
            assert_eq!(keyup.dw_flags, KEYEVENTF_KEYUP);
        }
    }

    #[cfg(all(test, windows))]
    #[test]
    fn ascii_key_input_sequence_uses_virtual_keys() {
        let sequence = ascii_key_input_sequence('e').unwrap();

        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence[0].input_type, INPUT_KEYBOARD);
        assert_eq!(sequence[1].input_type, INPUT_KEYBOARD);
        let down = unsafe { sequence[0].union.ki };
        let up = unsafe { sequence[1].union.ki };
        assert_eq!(down.w_vk, b'E' as u16);
        assert_eq!(down.w_scan, 0);
        assert_eq!(down.dw_flags, 0);
        assert_eq!(up.w_vk, b'E' as u16);
        assert_eq!(up.w_scan, 0);
        assert_eq!(up.dw_flags, KEYEVENTF_KEYUP);
    }

    #[cfg(all(test, windows))]
    #[test]
    fn ascii_key_input_sequence_presses_shift_for_uppercase() {
        let sequence = ascii_key_input_sequence('E').unwrap();

        assert_eq!(sequence.len(), 4);
        let shift_down = unsafe { sequence[0].union.ki };
        let key_down = unsafe { sequence[1].union.ki };
        let key_up = unsafe { sequence[2].union.ki };
        let shift_up = unsafe { sequence[3].union.ki };
        assert_eq!(shift_down.w_vk, VK_SHIFT);
        assert_eq!(shift_down.dw_flags, 0);
        assert_eq!(key_down.w_vk, b'E' as u16);
        assert_eq!(key_down.dw_flags, 0);
        assert_eq!(key_up.w_vk, b'E' as u16);
        assert_eq!(key_up.dw_flags, KEYEVENTF_KEYUP);
        assert_eq!(shift_up.w_vk, VK_SHIFT);
        assert_eq!(shift_up.dw_flags, KEYEVENTF_KEYUP);
    }

    #[cfg(all(test, windows, target_pointer_width = "64"))]
    #[test]
    fn windows_input_struct_matches_sendinput_size_on_x64() {
        assert_eq!(std::mem::size_of::<Input>(), 40);
    }

    #[cfg(all(test, windows, target_pointer_width = "32"))]
    #[test]
    fn windows_input_struct_matches_sendinput_size_on_x86() {
        assert_eq!(std::mem::size_of::<Input>(), 28);
    }
}

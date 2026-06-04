use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub accelerator: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleHotkeyBinding {
    pub accelerator: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHotkeyBindings {
    pub push_to_talk: String,
    pub toggle_dictation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyRegistrationStatus {
    pub accelerator: String,
    pub registered: bool,
    pub message: String,
}

impl HotkeyRegistrationStatus {
    pub fn registered(accelerator: String) -> Self {
        Self {
            message: format!("全局快捷键已注册：{accelerator}"),
            accelerator,
            registered: true,
        }
    }

    pub fn failed(accelerator: String, error: String) -> Self {
        Self {
            accelerator,
            registered: false,
            message: format!("全局快捷键注册失败：{error}"),
        }
    }
}

impl Default for HotkeyBinding {
    fn default() -> Self {
        Self {
            accelerator: "Ctrl+Alt+Space".to_string(),
        }
    }
}

impl Default for ToggleHotkeyBinding {
    fn default() -> Self {
        Self {
            accelerator: "Ctrl+Alt+V".to_string(),
        }
    }
}

pub fn runtime_bindings_from_preferences(
    preferences: &crate::preferences::UserPreferences,
) -> Result<RuntimeHotkeyBindings, String> {
    let push_to_talk = preferences
        .push_to_talk_hotkey
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&HotkeyBinding::default().accelerator)
        .to_string();
    let toggle_dictation = preferences
        .toggle_dictation_hotkey
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&ToggleHotkeyBinding::default().accelerator)
        .to_string();
    validate_hotkey_pair(&push_to_talk, &toggle_dictation)?;
    Ok(RuntimeHotkeyBindings {
        push_to_talk,
        toggle_dictation,
    })
}

pub fn validate_hotkey_pair(push_to_talk: &str, toggle: &str) -> Result<(), String> {
    let push = push_to_talk.trim();
    let toggle = toggle.trim();
    if push.is_empty() || toggle.is_empty() {
        return Err("快捷键不能为空".to_string());
    }
    if push.eq_ignore_ascii_case(toggle) {
        return Err("两个输入模式不能使用同一个快捷键".to_string());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushToTalkEvent {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushToTalkAction {
    StartRecording,
    StopAndTranscribe,
    ToggleStartRecording,
    ToggleStopAndTranscribe,
    Ignore,
}

#[derive(Debug, Default)]
pub struct PushToTalkState {
    pressed: bool,
}

#[derive(Debug, Default)]
pub struct ToggleDictationState;

impl PushToTalkState {
    pub fn handle_event(&mut self, event: PushToTalkEvent) -> PushToTalkAction {
        match (self.pressed, event) {
            (false, PushToTalkEvent::Pressed) => {
                self.pressed = true;
                PushToTalkAction::StartRecording
            }
            (true, PushToTalkEvent::Released) => {
                self.pressed = false;
                PushToTalkAction::StopAndTranscribe
            }
            _ => PushToTalkAction::Ignore,
        }
    }
}

impl ToggleDictationState {
    pub fn handle_pressed(&mut self) -> PushToTalkAction {
        PushToTalkAction::ToggleStartRecording
    }

    pub fn handle_released(&self) -> PushToTalkAction {
        PushToTalkAction::Ignore
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PushToTalkPayload {
    pub state: &'static str,
    pub action: &'static str,
}

impl PushToTalkAction {
    pub fn as_str(self) -> &'static str {
        match self {
            PushToTalkAction::StartRecording => "startRecording",
            PushToTalkAction::StopAndTranscribe => "stopAndTranscribe",
            PushToTalkAction::ToggleStartRecording => "toggleStartRecording",
            PushToTalkAction::ToggleStopAndTranscribe => "toggleStopAndTranscribe",
            PushToTalkAction::Ignore => "ignore",
        }
    }
}

impl PushToTalkEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            PushToTalkEvent::Pressed => "pressed",
            PushToTalkEvent::Released => "released",
        }
    }
}

pub fn payload_for_event(event: PushToTalkEvent, action: PushToTalkAction) -> PushToTalkPayload {
    PushToTalkPayload {
        state: event.as_str(),
        action: action.as_str(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressing_once_starts_recording() {
        let mut state = PushToTalkState::default();

        assert_eq!(
            state.handle_event(PushToTalkEvent::Pressed),
            PushToTalkAction::StartRecording
        );
    }

    #[test]
    fn repeated_press_is_ignored_until_release() {
        let mut state = PushToTalkState::default();

        assert_eq!(
            state.handle_event(PushToTalkEvent::Pressed),
            PushToTalkAction::StartRecording
        );
        assert_eq!(
            state.handle_event(PushToTalkEvent::Pressed),
            PushToTalkAction::Ignore
        );
    }

    #[test]
    fn release_after_press_stops_and_transcribes() {
        let mut state = PushToTalkState::default();

        state.handle_event(PushToTalkEvent::Pressed);

        assert_eq!(
            state.handle_event(PushToTalkEvent::Released),
            PushToTalkAction::StopAndTranscribe
        );
    }

    #[test]
    fn release_without_press_is_ignored() {
        let mut state = PushToTalkState::default();

        assert_eq!(
            state.handle_event(PushToTalkEvent::Released),
            PushToTalkAction::Ignore
        );
    }

    #[test]
    fn registration_status_records_success_and_failure() {
        let success = HotkeyRegistrationStatus::registered("Ctrl+Alt+Space".to_string());
        assert!(success.registered);
        assert_eq!(success.accelerator, "Ctrl+Alt+Space");
        assert_eq!(success.message, "全局快捷键已注册：Ctrl+Alt+Space");

        let failure = HotkeyRegistrationStatus::failed(
            "Ctrl+Alt+Space".to_string(),
            "shortcut already registered".to_string(),
        );
        assert!(!failure.registered);
        assert_eq!(
            failure.message,
            "全局快捷键注册失败：shortcut already registered"
        );
    }

    #[test]
    fn toggle_hotkey_press_is_frontend_toggle_signal() {
        let mut state = ToggleDictationState::default();

        assert_eq!(
            state.handle_pressed(),
            PushToTalkAction::ToggleStartRecording
        );
        assert_eq!(state.handle_released(), PushToTalkAction::Ignore);
        assert_eq!(
            state.handle_pressed(),
            PushToTalkAction::ToggleStartRecording
        );
    }

    #[test]
    fn toggle_binding_uses_separate_default_shortcut() {
        assert_eq!(ToggleHotkeyBinding::default().accelerator, "Ctrl+Alt+V");
        assert_ne!(
            ToggleHotkeyBinding::default().accelerator,
            HotkeyBinding::default().accelerator
        );
    }

    #[test]
    fn runtime_bindings_use_saved_preferences_with_defaults() {
        let preferences = crate::preferences::UserPreferences {
            selected_input_device_name: None,
            push_to_talk_hotkey: Some("Ctrl+Shift+Space".to_string()),
            toggle_dictation_hotkey: None,
            push_to_talk_model: crate::preferences::TranscriptionModelId::BaiduShort,
            toggle_dictation_model: crate::preferences::TranscriptionModelId::BaiduShort,
            insertion_strategy: crate::insertion::InsertionStrategyId::Clipboard,
        };

        let bindings = runtime_bindings_from_preferences(&preferences).unwrap();

        assert_eq!(bindings.push_to_talk, "Ctrl+Shift+Space");
        assert_eq!(bindings.toggle_dictation, "Ctrl+Alt+V");
    }

    #[test]
    fn hotkey_validation_rejects_duplicate_bindings() {
        let error = validate_hotkey_pair("Ctrl+Alt+Space", "Ctrl+Alt+Space").unwrap_err();

        assert_eq!(error, "两个输入模式不能使用同一个快捷键");
    }

    #[test]
    fn hotkey_validation_rejects_blank_bindings() {
        let error = validate_hotkey_pair("   ", "Ctrl+Alt+V").unwrap_err();

        assert_eq!(error, "快捷键不能为空");
    }

    #[test]
    fn hotkey_validation_accepts_distinct_bindings() {
        validate_hotkey_pair("Ctrl+Alt+Space", "Ctrl+Alt+V").unwrap();
    }
}

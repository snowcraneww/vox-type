use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub accelerator: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleHotkeyBinding {
    pub accelerator: String,
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
pub struct ToggleDictationState {
    recording: bool,
}

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
        if self.recording {
            self.recording = false;
            PushToTalkAction::ToggleStopAndTranscribe
        } else {
            self.recording = true;
            PushToTalkAction::ToggleStartRecording
        }
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
    fn toggle_hotkey_alternates_start_and_stop_on_presses() {
        let mut state = ToggleDictationState::default();

        assert_eq!(
            state.handle_pressed(),
            PushToTalkAction::ToggleStartRecording
        );
        assert_eq!(state.handle_released(), PushToTalkAction::Ignore);
        assert_eq!(
            state.handle_pressed(),
            PushToTalkAction::ToggleStopAndTranscribe
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
}

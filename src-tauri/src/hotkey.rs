use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub accelerator: String,
}

impl Default for HotkeyBinding {
    fn default() -> Self {
        Self {
            accelerator: "Ctrl+Alt+Space".to_string(),
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
    Ignore,
}

#[derive(Debug, Default)]
pub struct PushToTalkState {
    pressed: bool,
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
}

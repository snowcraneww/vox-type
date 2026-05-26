use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub phase: AppPhase,
    pub message: String,
    pub last_transcript: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AppPhase {
    Idle,
    Recording,
    Transcribing,
    Inserting,
    Succeeded,
    Failed,
}

impl AppStatus {
    pub fn idle() -> Self {
        Self {
            phase: AppPhase::Idle,
            message: "等待按住快捷键开始录音".to_string(),
            last_transcript: None,
        }
    }

    pub fn succeeded(transcript: String) -> Self {
        Self {
            phase: AppPhase::Succeeded,
            message: "模拟闭环完成".to_string(),
            last_transcript: Some(transcript),
        }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            phase: AppPhase::Failed,
            message: message.into(),
            last_transcript: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_status_has_no_transcript() {
        let status = AppStatus::idle();
        assert_eq!(status.phase, AppPhase::Idle);
        assert!(status.last_transcript.is_none());
    }
}

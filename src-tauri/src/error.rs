use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VoxError {
    #[error("快捷键注册失败：{0}")]
    Hotkey(String),
    #[error("麦克风不可用：{0}")]
    Recorder(String),
    #[error("模型不可用：{0}")]
    Model(String),
    #[error("语音识别失败：{0}")]
    Asr(String),
    #[error("文本上屏失败：{0}")]
    Insertion(String),
    #[error("配置错误：{0}")]
    Config(String),
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserFacingError {
    pub code: String,
    pub message: String,
}

impl From<VoxError> for UserFacingError {
    fn from(error: VoxError) -> Self {
        let code = match &error {
            VoxError::Hotkey(_) => "hotkey_failed",
            VoxError::Recorder(_) => "recorder_failed",
            VoxError::Model(_) => "model_failed",
            VoxError::Asr(_) => "asr_failed",
            VoxError::Insertion(_) => "insertion_failed",
            VoxError::Config(_) => "config_failed",
        };
        Self {
            code: code.to_string(),
            message: error.to_string(),
        }
    }
}

impl Serialize for VoxError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        UserFacingError::from(self.to_owned()).serialize(serializer)
    }
}

impl Clone for VoxError {
    fn clone(&self) -> Self {
        match self {
            Self::Hotkey(value) => Self::Hotkey(value.clone()),
            Self::Recorder(value) => Self::Recorder(value.clone()),
            Self::Model(value) => Self::Model(value.clone()),
            Self::Asr(value) => Self::Asr(value.clone()),
            Self::Insertion(value) => Self::Insertion(value.clone()),
            Self::Config(value) => Self::Config(value.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_error_to_user_facing_code() {
        let user_error = UserFacingError::from(VoxError::Insertion("剪贴板恢复失败".to_string()));
        assert_eq!(user_error.code, "insertion_failed");
        assert!(user_error.message.contains("剪贴板恢复失败"));
    }
}

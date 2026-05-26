use serde::{Deserialize, Serialize};

use crate::error::VoxError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub text: String,
    pub engine: String,
}

pub trait AsrEngine {
    fn transcribe(&self, pcm_16khz_mono: &[i16]) -> Result<Transcript, VoxError>;
}

#[derive(Debug, Default)]
pub struct MockAsrEngine;

impl AsrEngine for MockAsrEngine {
    fn transcribe(&self, _pcm_16khz_mono: &[i16]) -> Result<Transcript, VoxError> {
        Ok(Transcript {
            text: "这是 VoxType 的模拟转写结果。".to_string(),
            engine: "mock".to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhisperCppEngine {
    pub model_path: String,
}

impl AsrEngine for WhisperCppEngine {
    fn transcribe(&self, _pcm_16khz_mono: &[i16]) -> Result<Transcript, VoxError> {
        Err(VoxError::Asr(format!(
            "whisper.cpp 尚未接入，模型路径占位：{}",
            self.model_path
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_engine_returns_chinese_transcript() {
        let transcript = MockAsrEngine.transcribe(&[]).unwrap();
        assert_eq!(transcript.text, "这是 VoxType 的模拟转写结果。");
    }
}

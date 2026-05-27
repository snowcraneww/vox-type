use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

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
    pub binary_path: String,
    pub model_path: String,
    pub language: String,
}

impl AsrEngine for WhisperCppEngine {
    fn transcribe(&self, pcm_16khz_mono: &[i16]) -> Result<Transcript, VoxError> {
        let binary = PathBuf::from(&self.binary_path);
        if !binary.is_file() {
            return Err(VoxError::Asr(format!(
                "whisper.cpp 可执行文件不存在：{}",
                self.binary_path
            )));
        }

        let model = PathBuf::from(&self.model_path);
        if !model.is_file() {
            return Err(VoxError::Model(format!(
                "模型文件不存在：{}",
                self.model_path
            )));
        }

        let wav_path = temp_wav_path();
        write_pcm_wav(&wav_path, pcm_16khz_mono)?;

        let wav_path_string = wav_path.to_string_lossy().to_string();
        let args = build_whisper_args(&self.model_path, &wav_path_string, &self.language);
        let output = Command::new(&binary)
            .args(args)
            .output()
            .map_err(|error| VoxError::Asr(format!("启动 whisper.cpp 失败：{error}")))?;

        let _ = std::fs::remove_file(&wav_path);

        if !output.status.success() {
            return Err(VoxError::Asr(format!(
                "whisper.cpp 执行失败：{}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            return Err(VoxError::Asr("whisper.cpp 没有返回文本".to_string()));
        }

        Ok(Transcript {
            text,
            engine: "whisper.cpp".to_string(),
        })
    }
}

fn build_whisper_args(model_path: &str, wav_path: &str, language: &str) -> Vec<String> {
    let mut args = vec![
        "-m".to_string(),
        model_path.to_string(),
        "-f".to_string(),
        wav_path.to_string(),
        "-l".to_string(),
        language.to_string(),
        "-nt".to_string(),
    ];

    if language.to_ascii_lowercase().starts_with("zh") {
        args.push("--prompt".to_string());
        args.push("请使用简体中文输出。".to_string());
    }

    args
}
fn temp_wav_path() -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("voxtype-{now}.wav"))
}

pub fn write_pcm_wav(path: &PathBuf, pcm_16khz_mono: &[i16]) -> Result<(), VoxError> {
    let mut file =
        File::create(path).map_err(|error| VoxError::Asr(format!("创建临时 WAV 失败：{error}")))?;
    let data_len = (pcm_16khz_mono.len() * 2) as u32;
    let byte_rate = 16_000u32 * 2;

    file.write_all(b"RIFF").map_err(wav_error)?;
    file.write_all(&(36 + data_len).to_le_bytes())
        .map_err(wav_error)?;
    file.write_all(b"WAVEfmt ").map_err(wav_error)?;
    file.write_all(&16u32.to_le_bytes()).map_err(wav_error)?;
    file.write_all(&1u16.to_le_bytes()).map_err(wav_error)?;
    file.write_all(&1u16.to_le_bytes()).map_err(wav_error)?;
    file.write_all(&16_000u32.to_le_bytes())
        .map_err(wav_error)?;
    file.write_all(&byte_rate.to_le_bytes())
        .map_err(wav_error)?;
    file.write_all(&2u16.to_le_bytes()).map_err(wav_error)?;
    file.write_all(&16u16.to_le_bytes()).map_err(wav_error)?;
    file.write_all(b"data").map_err(wav_error)?;
    file.write_all(&data_len.to_le_bytes()).map_err(wav_error)?;
    for sample in pcm_16khz_mono {
        file.write_all(&sample.to_le_bytes()).map_err(wav_error)?;
    }
    file.seek(SeekFrom::Start(0)).map_err(wav_error)?;
    Ok(())
}

fn wav_error(error: std::io::Error) -> VoxError {
    VoxError::Asr(format!("写入临时 WAV 失败：{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_engine_returns_chinese_transcript() {
        let transcript = MockAsrEngine.transcribe(&[]).unwrap();
        assert_eq!(transcript.text, "这是 VoxType 的模拟转写结果。");
    }

    #[test]
    fn whisper_engine_reports_missing_binary_before_running() {
        let engine = WhisperCppEngine {
            binary_path: "missing-whisper-cli.exe".to_string(),
            model_path: "model.bin".to_string(),
            language: "zh".to_string(),
        };

        let error = engine.transcribe(&[0, 1, -1]).unwrap_err();

        assert!(error.to_string().contains("whisper.cpp 可执行文件不存在"));
    }

    #[test]
    fn whisper_engine_builds_temp_wav_path_with_wav_extension() {
        let path = temp_wav_path();
        assert_eq!(
            path.extension().and_then(|value| value.to_str()),
            Some("wav")
        );
    }

    #[test]
    fn whisper_args_include_simplified_chinese_prompt_for_zh() {
        let args = build_whisper_args("model.bin", "input.wav", "zh");

        assert!(args.contains(&"--prompt".to_string()));
        assert!(args.contains(&"请使用简体中文输出。".to_string()));
    }
    #[test]
    fn write_pcm_wav_creates_16khz_mono_header() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();

        write_pcm_wav(&path, &[0, 1000, -1000]).unwrap();
        let bytes = std::fs::read(path).unwrap();

        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(
            u32::from_le_bytes(bytes[24..28].try_into().unwrap()),
            16_000
        );
        assert_eq!(u16::from_le_bytes(bytes[34..36].try_into().unwrap()), 16);
        assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()), 6);
    }
}

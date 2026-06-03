use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::error::VoxError;

const CONFIG_FILE_NAME: &str = "sensevoice-config.json";
const DEFAULT_LANGUAGE: &str = "zh";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SenseVoiceConfig {
    pub runtime_path: Option<String>,
    pub model_path: Option<String>,
    pub tokens_path: Option<String>,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SenseVoiceConfigStatus {
    pub config: SenseVoiceConfig,
    pub runtime_configured: bool,
    pub model_configured: bool,
    pub tokens_configured: bool,
    pub runtime_exists: bool,
    pub model_exists: bool,
    pub tokens_exists: bool,
    pub ready: bool,
    pub source: String,
    pub message: String,
}

impl Default for SenseVoiceConfig {
    fn default() -> Self {
        Self {
            runtime_path: None,
            model_path: None,
            tokens_path: None,
            language: DEFAULT_LANGUAGE.to_string(),
        }
    }
}

pub fn load_sensevoice_config(config_dir: PathBuf) -> SenseVoiceConfig {
    fs::read_to_string(config_dir.join(CONFIG_FILE_NAME))
        .ok()
        .and_then(|payload| serde_json::from_str::<SenseVoiceConfig>(&payload).ok())
        .map(normalize_config)
        .unwrap_or_default()
}

pub fn save_sensevoice_config(
    config_dir: PathBuf,
    config: SenseVoiceConfig,
) -> Result<SenseVoiceConfigStatus, VoxError> {
    let normalized = normalize_config(config);
    fs::create_dir_all(&config_dir)
        .map_err(|error| VoxError::Config(format!("创建 SenseVoice 配置目录失败：{error}")))?;
    let payload = serde_json::to_string_pretty(&normalized)
        .map_err(|error| VoxError::Config(format!("序列化 SenseVoice 配置失败：{error}")))?;
    fs::write(config_dir.join(CONFIG_FILE_NAME), payload)
        .map_err(|error| VoxError::Config(format!("写入 SenseVoice 配置失败：{error}")))?;
    Ok(status_from_config(normalized, "app".to_string()))
}

pub fn get_sensevoice_config_status(config_dir: PathBuf) -> SenseVoiceConfigStatus {
    let path = config_dir.join(CONFIG_FILE_NAME);
    let source = if path.is_file() { "app" } else { "default" };
    status_from_config(load_sensevoice_config(config_dir), source.to_string())
}

pub fn status_from_config(config: SenseVoiceConfig, source: String) -> SenseVoiceConfigStatus {
    let config = normalize_config(config);
    let runtime_configured = config.runtime_path.is_some();
    let model_configured = config.model_path.is_some();
    let tokens_configured = config.tokens_path.is_some();
    let runtime_exists = config
        .runtime_path
        .as_ref()
        .map(|path| PathBuf::from(path).is_file() && !is_deprecated_gui_runtime(path))
        .unwrap_or(false);
    let model_exists = config
        .model_path
        .as_ref()
        .map(|path| PathBuf::from(path).is_file())
        .unwrap_or(false);
    let tokens_exists = config
        .tokens_path
        .as_ref()
        .map(|path| PathBuf::from(path).is_file())
        .unwrap_or(false);
    let ready = runtime_exists && model_exists && tokens_exists;
    let message = status_message(
        runtime_configured,
        model_configured,
        tokens_configured,
        runtime_exists,
        model_exists,
        tokens_exists,
    );
    SenseVoiceConfigStatus {
        config,
        runtime_configured,
        model_configured,
        tokens_configured,
        runtime_exists,
        model_exists,
        tokens_exists,
        ready,
        source,
        message,
    }
}

pub fn normalize_config(config: SenseVoiceConfig) -> SenseVoiceConfig {
    SenseVoiceConfig {
        runtime_path: normalize_optional(config.runtime_path),
        model_path: normalize_optional(config.model_path),
        tokens_path: normalize_optional(config.tokens_path),
        language: normalize_with_default(config.language, DEFAULT_LANGUAGE),
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn normalize_with_default(value: String, default_value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default_value.to_string()
    } else {
        trimmed.to_string()
    }
}

fn is_deprecated_gui_runtime(path: &str) -> bool {
    path.replace('\\', "/")
        .ends_with("sherpa-onnx-non-streaming-asr-x64-v1.13.2.exe")
}

fn status_message(
    runtime_configured: bool,
    model_configured: bool,
    tokens_configured: bool,
    runtime_exists: bool,
    model_exists: bool,
    tokens_exists: bool,
) -> String {
    if runtime_exists && model_exists && tokens_exists {
        return "SenseVoice Small 本地运行时已就绪。".to_string();
    }
    let mut problems = Vec::new();
    if !runtime_configured {
        problems.push("缺少 sherpa-onnx 本地运行时");
    } else if !runtime_exists {
        problems.push("sherpa-onnx 本地运行时不存在");
    }
    if !model_configured {
        problems.push("缺少 SenseVoice ONNX 模型文件");
    } else if !model_exists {
        problems.push("SenseVoice ONNX 模型文件不存在");
    }
    if !tokens_configured {
        problems.push("缺少 tokens.txt 文件");
    } else if !tokens_exists {
        problems.push("tokens.txt 文件不存在");
    }
    format!("SenseVoice Small 未就绪：{}。", problems.join("；"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_managed_sherpa_runtime_shape() {
        let config = SenseVoiceConfig::default();
        assert_eq!(config.runtime_path, None);
        assert_eq!(config.model_path, None);
        assert_eq!(config.tokens_path, None);
        assert_eq!(config.language, "zh");
    }

    #[test]
    fn status_requires_runtime_model_and_tokens() {
        let status = status_from_config(SenseVoiceConfig::default(), "default".to_string());
        assert!(!status.ready);
        assert!(status.message.contains("sherpa-onnx"));
        assert!(status.message.contains("ONNX"));
        assert!(status.message.contains("tokens"));
    }

    #[test]
    fn status_rejects_deprecated_gui_runtime_even_when_file_exists() {
        let runtime = tempfile::tempdir().unwrap();
        let runtime_path = runtime
            .path()
            .join("sherpa-onnx-non-streaming-asr-x64-v1.13.2.exe");
        fs::write(&runtime_path, b"gui").unwrap();
        let model = tempfile::NamedTempFile::new().unwrap();
        let tokens = tempfile::NamedTempFile::new().unwrap();

        let status = status_from_config(
            SenseVoiceConfig {
                runtime_path: Some(runtime_path.to_string_lossy().to_string()),
                model_path: Some(model.path().to_string_lossy().to_string()),
                tokens_path: Some(tokens.path().to_string_lossy().to_string()),
                language: "zh".to_string(),
            },
            "app".to_string(),
        );

        assert!(!status.ready);
        assert!(!status.runtime_exists);
    }

    #[test]
    fn status_reports_existing_runtime_model_and_tokens_as_ready() {
        let runtime = tempfile::NamedTempFile::new().unwrap();
        let model = tempfile::NamedTempFile::new().unwrap();
        let tokens = tempfile::NamedTempFile::new().unwrap();
        let status = status_from_config(
            SenseVoiceConfig {
                runtime_path: Some(runtime.path().to_string_lossy().to_string()),
                model_path: Some(model.path().to_string_lossy().to_string()),
                tokens_path: Some(tokens.path().to_string_lossy().to_string()),
                language: " auto ".to_string(),
            },
            "app".to_string(),
        );
        assert!(status.ready);
        assert_eq!(status.config.language, "auto");
        assert_eq!(status.message, "SenseVoice Small 本地运行时已就绪。");
    }
}

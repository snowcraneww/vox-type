use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::error::VoxError;

const CONFIG_FILE_NAME: &str = "asr-config.json";
const ENV_BINARY: &str = "VOXTYPE_WHISPER_CPP_BINARY";
const ENV_MODEL: &str = "VOXTYPE_WHISPER_CPP_MODEL";
const ENV_LANGUAGE: &str = "VOXTYPE_ASR_LANGUAGE";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AsrConfig {
    pub whisper_binary_path: Option<String>,
    pub whisper_model_path: Option<String>,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AsrConfigStatus {
    pub whisper_binary_path: Option<String>,
    pub whisper_model_path: Option<String>,
    pub language: String,
    pub binary_configured: bool,
    pub model_configured: bool,
    pub binary_exists: bool,
    pub model_exists: bool,
    pub ready: bool,
    pub source: String,
    pub message: String,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            whisper_binary_path: None,
            whisper_model_path: None,
            language: "zh".to_string(),
        }
    }
}

pub fn load_asr_config(config_dir: PathBuf) -> AsrConfig {
    let app_config = read_app_config(config_dir.join(CONFIG_FILE_NAME));
    merge_with_env(app_config)
}

pub fn save_asr_config(
    config_dir: PathBuf,
    config: AsrConfig,
) -> Result<AsrConfigStatus, VoxError> {
    let normalized = normalize_config(config);
    fs::create_dir_all(&config_dir)
        .map_err(|error| VoxError::Config(format!("创建配置目录失败：{error}")))?;
    let path = config_dir.join(CONFIG_FILE_NAME);
    let payload = serde_json::to_string_pretty(&normalized)
        .map_err(|error| VoxError::Config(format!("序列化 ASR 配置失败：{error}")))?;
    fs::write(path, payload)
        .map_err(|error| VoxError::Config(format!("写入 ASR 配置失败：{error}")))?;
    Ok(status_from_config(normalized, "app".to_string()))
}

pub fn get_asr_config_status(config_dir: PathBuf) -> AsrConfigStatus {
    let path = config_dir.join(CONFIG_FILE_NAME);
    let source = if path.is_file() {
        "app"
    } else if std::env::var(ENV_BINARY).is_ok() || std::env::var(ENV_MODEL).is_ok() {
        "env"
    } else {
        "default"
    };
    status_from_config(load_asr_config(config_dir), source.to_string())
}

pub fn status_from_config(config: AsrConfig, source: String) -> AsrConfigStatus {
    let binary_configured = config.whisper_binary_path.is_some();
    let model_configured = config.whisper_model_path.is_some();
    let binary_exists = config
        .whisper_binary_path
        .as_ref()
        .map(|path| PathBuf::from(path).is_file())
        .unwrap_or(false);
    let model_exists = config
        .whisper_model_path
        .as_ref()
        .map(|path| PathBuf::from(path).is_file())
        .unwrap_or(false);
    let ready = binary_exists && model_exists;
    let message = status_message(
        binary_configured,
        model_configured,
        binary_exists,
        model_exists,
    );

    AsrConfigStatus {
        whisper_binary_path: config.whisper_binary_path,
        whisper_model_path: config.whisper_model_path,
        language: config.language,
        binary_configured,
        model_configured,
        binary_exists,
        model_exists,
        ready,
        source,
        message,
    }
}

fn read_app_config(path: PathBuf) -> Option<AsrConfig> {
    let payload = fs::read_to_string(path).ok()?;
    serde_json::from_str(&payload).ok().map(normalize_config)
}

fn merge_with_env(app_config: Option<AsrConfig>) -> AsrConfig {
    let mut config = app_config.unwrap_or_default();
    if config.whisper_binary_path.is_none() {
        config.whisper_binary_path = read_env(ENV_BINARY);
    }
    if config.whisper_model_path.is_none() {
        config.whisper_model_path = read_env(ENV_MODEL);
    }
    if config.language.trim().is_empty() || config.language == "zh" {
        config.language = read_env(ENV_LANGUAGE).unwrap_or(config.language);
    }
    normalize_config(config)
}

fn normalize_config(config: AsrConfig) -> AsrConfig {
    AsrConfig {
        whisper_binary_path: normalize_optional_path(config.whisper_binary_path),
        whisper_model_path: normalize_optional_path(config.whisper_model_path),
        language: normalize_language(config.language),
    }
}

fn normalize_optional_path(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn normalize_language(value: String) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "zh".to_string()
    } else {
        trimmed.to_string()
    }
}

fn read_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn status_message(
    binary_configured: bool,
    model_configured: bool,
    binary_exists: bool,
    model_exists: bool,
) -> String {
    if binary_exists && model_exists {
        return "ASR 配置已就绪，可以调用 whisper.cpp。".to_string();
    }

    let mut problems = Vec::new();
    if !binary_configured {
        problems.push("缺少 whisper.cpp 可执行文件路径");
    } else if !binary_exists {
        problems.push("whisper.cpp 可执行文件路径不存在");
    }
    if !model_configured {
        problems.push("缺少模型文件路径");
    } else if !model_exists {
        problems.push("模型文件路径不存在");
    }

    format!("ASR 配置未就绪：{}。", problems.join("；"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_reports_missing_paths() {
        let status = status_from_config(
            AsrConfig {
                whisper_binary_path: None,
                whisper_model_path: None,
                language: "zh".to_string(),
            },
            "default".to_string(),
        );

        assert!(!status.ready);
        assert!(!status.binary_configured);
        assert!(!status.model_configured);
        assert!(status.message.contains("whisper.cpp 可执行文件"));
        assert!(status.message.contains("模型文件"));
    }

    #[test]
    fn status_reports_existing_files_as_ready() {
        let binary = tempfile::NamedTempFile::new().unwrap();
        let model = tempfile::NamedTempFile::new().unwrap();

        let status = status_from_config(
            AsrConfig {
                whisper_binary_path: Some(binary.path().to_string_lossy().to_string()),
                whisper_model_path: Some(model.path().to_string_lossy().to_string()),
                language: "zh".to_string(),
            },
            "app".to_string(),
        );

        assert!(status.ready);
        assert!(status.binary_exists);
        assert!(status.model_exists);
        assert_eq!(status.message, "ASR 配置已就绪，可以调用 whisper.cpp。")
    }

    #[test]
    fn status_reports_configured_but_missing_files() {
        let status = status_from_config(
            AsrConfig {
                whisper_binary_path: Some("C:/missing/whisper-cli.exe".to_string()),
                whisper_model_path: Some("C:/missing/ggml-model.bin".to_string()),
                language: "zh".to_string(),
            },
            "app".to_string(),
        );

        assert!(!status.ready);
        assert!(status.binary_configured);
        assert!(status.model_configured);
        assert!(!status.binary_exists);
        assert!(!status.model_exists);
        assert!(status.message.contains("路径不存在"));
    }

    #[test]
    fn saves_and_loads_app_config_from_config_dir() {
        let dir = tempfile::tempdir().unwrap();
        let binary = tempfile::NamedTempFile::new().unwrap();
        let model = tempfile::NamedTempFile::new().unwrap();

        let saved = save_asr_config(
            dir.path().to_path_buf(),
            AsrConfig {
                whisper_binary_path: Some(binary.path().to_string_lossy().to_string()),
                whisper_model_path: Some(model.path().to_string_lossy().to_string()),
                language: "zh".to_string(),
            },
        )
        .unwrap();
        let loaded = load_asr_config(dir.path().to_path_buf());

        assert!(saved.ready);
        assert_eq!(
            loaded.whisper_binary_path,
            Some(binary.path().to_string_lossy().to_string())
        );
        assert_eq!(
            loaded.whisper_model_path,
            Some(model.path().to_string_lossy().to_string())
        );
        assert_eq!(loaded.language, "zh");
    }
}

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::error::VoxError;

const PREFERENCES_FILE_NAME: &str = "user-preferences.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TranscriptionModelId {
    LocalWhisper,
    BaiduShort,
    BaiduRealtime,
}

fn default_transcription_model() -> TranscriptionModelId {
    TranscriptionModelId::BaiduShort
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    pub selected_input_device_name: Option<String>,
    pub push_to_talk_hotkey: Option<String>,
    pub toggle_dictation_hotkey: Option<String>,
    #[serde(default = "default_transcription_model")]
    pub push_to_talk_model: TranscriptionModelId,
    #[serde(default = "default_transcription_model")]
    pub toggle_dictation_model: TranscriptionModelId,
}

impl Default for TranscriptionModelId {
    fn default() -> Self {
        default_transcription_model()
    }
}

pub fn load_user_preferences(config_dir: PathBuf) -> UserPreferences {
    let path = config_dir.join(PREFERENCES_FILE_NAME);
    let Some(preferences) = fs::read_to_string(path)
        .ok()
        .and_then(|payload| serde_json::from_str::<UserPreferences>(&payload).ok())
    else {
        return UserPreferences::default();
    };

    normalize_preferences(preferences)
}

pub fn save_user_preferences(
    config_dir: PathBuf,
    preferences: UserPreferences,
) -> Result<UserPreferences, VoxError> {
    let normalized = normalize_preferences(preferences);
    fs::create_dir_all(&config_dir)
        .map_err(|error| VoxError::Config(format!("创建偏好配置目录失败：{error}")))?;
    let payload = serde_json::to_string_pretty(&normalized)
        .map_err(|error| VoxError::Config(format!("序列化偏好配置失败：{error}")))?;
    fs::write(config_dir.join(PREFERENCES_FILE_NAME), payload)
        .map_err(|error| VoxError::Config(format!("写入偏好配置失败：{error}")))?;
    Ok(normalized)
}

fn normalize_preferences(preferences: UserPreferences) -> UserPreferences {
    UserPreferences {
        selected_input_device_name: preferences
            .selected_input_device_name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        push_to_talk_hotkey: preferences
            .push_to_talk_hotkey
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        toggle_dictation_hotkey: preferences
            .toggle_dictation_hotkey
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        push_to_talk_model: preferences.push_to_talk_model,
        toggle_dictation_model: preferences.toggle_dictation_model,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_selected_input_device_name() {
        let dir = tempfile::tempdir().unwrap();

        save_user_preferences(
            dir.path().to_path_buf(),
            UserPreferences {
                selected_input_device_name: Some("Studio Microphone".to_string()),
                push_to_talk_hotkey: None,
                toggle_dictation_hotkey: None,
                push_to_talk_model: TranscriptionModelId::BaiduShort,
                toggle_dictation_model: TranscriptionModelId::BaiduShort,
            },
        )
        .unwrap();

        let loaded = load_user_preferences(dir.path().to_path_buf());

        assert_eq!(
            loaded.selected_input_device_name.as_deref(),
            Some("Studio Microphone")
        );
    }

    #[test]
    fn saves_and_loads_hotkey_preferences() {
        let dir = tempfile::tempdir().unwrap();

        save_user_preferences(
            dir.path().to_path_buf(),
            UserPreferences {
                selected_input_device_name: None,
                push_to_talk_hotkey: Some("Ctrl+Alt+Space".to_string()),
                toggle_dictation_hotkey: Some("Ctrl+Alt+V".to_string()),
                push_to_talk_model: TranscriptionModelId::BaiduShort,
                toggle_dictation_model: TranscriptionModelId::BaiduShort,
            },
        )
        .unwrap();

        let loaded = load_user_preferences(dir.path().to_path_buf());

        assert_eq!(
            loaded.push_to_talk_hotkey.as_deref(),
            Some("Ctrl+Alt+Space")
        );
        assert_eq!(
            loaded.toggle_dictation_hotkey.as_deref(),
            Some("Ctrl+Alt+V")
        );
    }

    #[test]
    fn trims_blank_selected_input_device_name_to_none() {
        let dir = tempfile::tempdir().unwrap();

        let saved = save_user_preferences(
            dir.path().to_path_buf(),
            UserPreferences {
                selected_input_device_name: Some("   ".to_string()),
                push_to_talk_hotkey: Some("   ".to_string()),
                toggle_dictation_hotkey: Some("   ".to_string()),
                push_to_talk_model: TranscriptionModelId::BaiduShort,
                toggle_dictation_model: TranscriptionModelId::BaiduShort,
            },
        )
        .unwrap();

        assert_eq!(saved.selected_input_device_name, None);
        assert_eq!(saved.push_to_talk_hotkey, None);
        assert_eq!(saved.toggle_dictation_hotkey, None);
    }

    #[test]
    fn preserves_hotkeys_when_selected_device_changes() {
        let dir = tempfile::tempdir().unwrap();

        let mut preferences = UserPreferences {
            selected_input_device_name: None,
            push_to_talk_hotkey: Some("Ctrl+Shift+Space".to_string()),
            toggle_dictation_hotkey: Some("Ctrl+Shift+V".to_string()),
            push_to_talk_model: TranscriptionModelId::LocalWhisper,
            toggle_dictation_model: TranscriptionModelId::BaiduShort,
        };
        preferences.selected_input_device_name = Some("Remote Audio".to_string());

        let saved = save_user_preferences(dir.path().to_path_buf(), preferences).unwrap();

        assert_eq!(
            saved.selected_input_device_name.as_deref(),
            Some("Remote Audio")
        );
        assert_eq!(
            saved.push_to_talk_hotkey.as_deref(),
            Some("Ctrl+Shift+Space")
        );
        assert_eq!(
            saved.toggle_dictation_hotkey.as_deref(),
            Some("Ctrl+Shift+V")
        );
        assert_eq!(saved.push_to_talk_model, TranscriptionModelId::LocalWhisper);
        assert_eq!(
            saved.toggle_dictation_model,
            TranscriptionModelId::BaiduShort
        );
    }

    #[test]
    fn defaults_mode_models_to_baidu_short() {
        let loaded = serde_json::from_str::<UserPreferences>(r#"{}"#).unwrap();

        assert_eq!(loaded.push_to_talk_model, TranscriptionModelId::BaiduShort);
        assert_eq!(
            loaded.toggle_dictation_model,
            TranscriptionModelId::BaiduShort
        );
    }

    #[test]
    fn saves_and_loads_mode_model_preferences() {
        let dir = tempfile::tempdir().unwrap();

        save_user_preferences(
            dir.path().to_path_buf(),
            UserPreferences {
                selected_input_device_name: None,
                push_to_talk_hotkey: None,
                toggle_dictation_hotkey: None,
                push_to_talk_model: TranscriptionModelId::LocalWhisper,
                toggle_dictation_model: TranscriptionModelId::BaiduRealtime,
            },
        )
        .unwrap();

        let loaded = load_user_preferences(dir.path().to_path_buf());

        assert_eq!(
            loaded.push_to_talk_model,
            TranscriptionModelId::LocalWhisper
        );
        assert_eq!(
            loaded.toggle_dictation_model,
            TranscriptionModelId::BaiduRealtime
        );
    }
}

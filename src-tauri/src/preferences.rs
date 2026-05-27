use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::error::VoxError;

const PREFERENCES_FILE_NAME: &str = "user-preferences.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    pub selected_input_device_name: Option<String>,
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
    fn trims_blank_selected_input_device_name_to_none() {
        let dir = tempfile::tempdir().unwrap();

        let saved = save_user_preferences(
            dir.path().to_path_buf(),
            UserPreferences {
                selected_input_device_name: Some("   ".to_string()),
            },
        )
        .unwrap();

        assert_eq!(saved.selected_input_device_name, None);
    }
}

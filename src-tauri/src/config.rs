use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub hotkey: String,
    pub language: String,
    pub asr_engine: String,
    pub insertion_strategy: String,
    pub show_status_indicator: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+Alt+Space".to_string(),
            language: "zh-CN".to_string(),
            asr_engine: "whisper.cpp".to_string(),
            insertion_strategy: "clipboard".to_string(),
            show_status_indicator: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_confirmed_direction() {
        let config = AppConfig::default();
        assert_eq!(config.language, "zh-CN");
        assert_eq!(config.asr_engine, "whisper.cpp");
        assert_eq!(config.insertion_strategy, "clipboard");
    }
}

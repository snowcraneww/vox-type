use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::VoxError;

const POSTPROCESS_CONFIG_FILE: &str = "transcript-postprocess.json";

pub fn load_transcript_postprocess_config(config_dir: PathBuf) -> TranscriptPostprocessConfig {
    let path = postprocess_config_path(&config_dir);
    let Ok(text) = std::fs::read_to_string(path) else {
        return TranscriptPostprocessConfig::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save_transcript_postprocess_config(config_dir: PathBuf, mut config: TranscriptPostprocessConfig) -> Result<TranscriptPostprocessConfig, VoxError> {
    config.replacements.retain(|rule| !rule.from.trim().is_empty());
    for rule in &mut config.replacements {
        rule.from = rule.from.trim().to_string();
        rule.to = rule.to.trim().to_string();
    }
    config.glossary = config.glossary.into_iter().map(|term| term.trim().to_string()).filter(|term| !term.is_empty()).collect();
    std::fs::create_dir_all(&config_dir).map_err(|error| VoxError::Config(format!("failed to create postprocess config directory: {error}")))?;
    let text = serde_json::to_string_pretty(&config).map_err(|error| VoxError::Config(format!("failed to serialize postprocess config: {error}")))?;
    std::fs::write(postprocess_config_path(&config_dir), text).map_err(|error| VoxError::Config(format!("failed to write postprocess config: {error}")))?;
    Ok(config)
}

fn postprocess_config_path(config_dir: &Path) -> std::path::PathBuf {
    config_dir.join(POSTPROCESS_CONFIG_FILE)
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplacementRule {
    pub from: String,
    pub to: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptPostprocessConfig {
    pub enabled: bool,
    pub cleanup_noise: bool,
    pub glossary: Vec<String>,
    pub replacements: Vec<ReplacementRule>,
}

impl Default for TranscriptPostprocessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cleanup_noise: true,
            glossary: vec!["WebSocket".to_string(), "whisper.cpp".to_string(), "VoxType".to_string()],
            replacements: vec![ReplacementRule { from: "scale".to_string(), to: "skill".to_string(), enabled: true }],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PostprocessResult {
    pub text: String,
    pub rules_applied: usize,
    pub noise_removed: bool,
}

pub fn process_transcript(text: &str, config: &TranscriptPostprocessConfig) -> PostprocessResult {
    let original = normalize_whitespace(text);
    if !config.enabled {
        return PostprocessResult { text: original, rules_applied: 0, noise_removed: false };
    }

    if config.cleanup_noise && is_noise_only(&original) {
        return PostprocessResult { text: String::new(), rules_applied: 0, noise_removed: true };
    }

    let mut output = original;
    let mut rules: Vec<&ReplacementRule> = config.replacements.iter().filter(|rule| rule.enabled && !rule.from.trim().is_empty()).collect();
    rules.sort_by_key(|rule| std::cmp::Reverse(rule.from.chars().count()));
    let mut rules_applied = 0;
    for rule in rules {
        let next = replace_case_insensitive(&output, rule.from.trim(), rule.to.trim());
        if next != output {
            rules_applied += 1;
            output = next;
        }
    }

    for term in &config.glossary {
        output = apply_glossary_term(&output, term);
    }
    output = normalize_whitespace(&output);

    PostprocessResult { text: output, rules_applied, noise_removed: false }
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
}

fn is_noise_only(text: &str) -> bool {
    let trimmed = text.trim();
    let unwrapped = trimmed
        .trim_start_matches(['(', '[', '{'])
        .trim_start_matches(char::is_whitespace)
        .trim_start_matches('\u{ff08}')
        .trim_end_matches([')', ']', '}'])
        .trim_end_matches(char::is_whitespace)
        .trim_end_matches('\u{ff09}')
        .trim();
    let lower = unwrapped.to_lowercase();
    ["j chong", "caption:j chong", "subtitle:j chong"].iter().any(|pattern| lower.contains(pattern))
        || unwrapped.contains("J Chong")
        || unwrapped == "\u{4e0d}\u{77e5}"
        || unwrapped == "\u{7121}\u{6cd5} \u{5bf6}\u{5bf6}"
        || unwrapped == "\u{65e0}\u{6cd5} \u{5b9d}\u{5b9d}"
}

fn replace_case_insensitive(input: &str, from: &str, to: &str) -> String {
    if from.is_empty() {
        return input.to_string();
    }
    let lower_input = input.to_lowercase();
    let lower_from = from.to_lowercase();
    let mut output = String::new();
    let mut cursor = 0;
    while let Some(relative) = lower_input[cursor..].find(&lower_from) {
        let start = cursor + relative;
        let end = start + lower_from.len();
        output.push_str(&input[cursor..start]);
        output.push_str(to);
        cursor = end;
    }
    output.push_str(&input[cursor..]);
    output
}

fn apply_glossary_term(input: &str, term: &str) -> String {
    match term {
        "WebSocket" => replace_case_insensitive(input, "websocket", "WebSocket"),
        "VoxType" => replace_case_insensitive(input, "voxtype", "VoxType"),
        "whisper.cpp" => {
            let first = replace_case_insensitive(input, "whisper cpp", "whisper.cpp");
            replace_case_insensitive(&first, "whisper.cpp", "whisper.cpp")
        }
        _ => replace_case_insensitive(input, term, term),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TranscriptPostprocessConfig {
        TranscriptPostprocessConfig {
            enabled: true,
            cleanup_noise: true,
            glossary: vec!["WebSocket".to_string(), "whisper.cpp".to_string(), "VoxType".to_string()],
            replacements: vec![ReplacementRule { from: "scale".to_string(), to: "skill".to_string(), enabled: true }],
        }
    }

    #[test]
    fn applies_replacement_rules() {
        let result = process_transcript("open the scale panel", &config());
        assert_eq!(result.text, "open the skill panel");
        assert_eq!(result.rules_applied, 1);
    }

    #[test]
    fn applies_longest_replacement_first() {
        let mut cfg = config();
        cfg.replacements = vec![
            ReplacementRule { from: "scale".to_string(), to: "skill".to_string(), enabled: true },
            ReplacementRule { from: "scale bar".to_string(), to: "skill bar".to_string(), enabled: true },
        ];
        let result = process_transcript("open the scale bar", &cfg);
        assert_eq!(result.text, "open the skill bar");
        assert_eq!(result.rules_applied, 1);
    }

    #[test]
    fn normalizes_glossary_spelling_and_whitespace() {
        let result = process_transcript("  voxtype uses websocket and whisper cpp  ", &config());
        assert_eq!(result.text, "VoxType uses WebSocket and whisper.cpp");
    }

    #[test]
    fn drops_known_noise_only_text() {
        let noise = format!("({}:J Chong)", "\u{5b57}\u{5e55}");
        let result = process_transcript(&noise, &config());
        assert_eq!(result.text, "");
        assert_eq!(result.noise_removed, true);
    }
}


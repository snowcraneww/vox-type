use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::audio_preprocess::AudioPreprocessSummary;
use crate::audio_quality::AudioQualitySummary;
use crate::error::VoxError;

const HISTORY_FILE: &str = "transcript-history.json";
const MAX_HISTORY_ENTRIES: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedTranscriptEntry {
    pub id: String,
    pub text: String,
    pub input_mode: String,
    pub model: String,
    pub created_at_ms: u64,
    pub duration_ms: u64,
    pub character_count: usize,
    pub postprocess_rules_applied: usize,
    pub audio_quality: Option<AudioQualitySummary>,
    pub audio_preprocess: Option<AudioPreprocessSummary>,
}

pub fn load_transcript_history(
    config_dir: PathBuf,
) -> Result<Vec<PersistedTranscriptEntry>, VoxError> {
    let path = history_path(&config_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|error| VoxError::Config(format!("failed to read transcript history: {error}")))?;
    match serde_json::from_str::<Vec<PersistedTranscriptEntry>>(&text) {
        Ok(entries) => Ok(entries),
        Err(_) => Ok(Vec::new()),
    }
}

pub fn save_transcript_history_entry(
    config_dir: PathBuf,
    entry: PersistedTranscriptEntry,
) -> Result<Vec<PersistedTranscriptEntry>, VoxError> {
    if entry.id.trim().is_empty() || entry.text.trim().is_empty() {
        return Err(VoxError::Config(
            "transcript history entry requires id and text".to_string(),
        ));
    }
    let mut entries = load_transcript_history(config_dir.clone())?;
    entries.retain(|existing| existing.id != entry.id);
    entries.insert(0, entry);
    entries.truncate(MAX_HISTORY_ENTRIES);
    write_history(&config_dir, &entries)?;
    Ok(entries)
}

pub fn delete_transcript_history_entry(
    config_dir: PathBuf,
    id: String,
) -> Result<Vec<PersistedTranscriptEntry>, VoxError> {
    let mut entries = load_transcript_history(config_dir.clone())?;
    entries.retain(|entry| entry.id != id);
    write_history(&config_dir, &entries)?;
    Ok(entries)
}

pub fn clear_transcript_history(
    config_dir: PathBuf,
) -> Result<Vec<PersistedTranscriptEntry>, VoxError> {
    write_history(&config_dir, &[])?;
    Ok(Vec::new())
}

fn history_path(config_dir: &Path) -> PathBuf {
    config_dir.join(HISTORY_FILE)
}

fn write_history(config_dir: &Path, entries: &[PersistedTranscriptEntry]) -> Result<(), VoxError> {
    std::fs::create_dir_all(config_dir).map_err(|error| {
        VoxError::Config(format!(
            "failed to create transcript history directory: {error}"
        ))
    })?;
    let text = serde_json::to_string_pretty(entries).map_err(|error| {
        VoxError::Config(format!("failed to serialize transcript history: {error}"))
    })?;
    std::fs::write(history_path(config_dir), text)
        .map_err(|error| VoxError::Config(format!("failed to write transcript history: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_quality::{AudioQualitySummary, AudioQualityWarning};

    fn entry(id: &str) -> PersistedTranscriptEntry {
        PersistedTranscriptEntry {
            id: id.to_string(),
            text: "hello".to_string(),
            input_mode: "push-to-talk".to_string(),
            model: "baidu-short".to_string(),
            created_at_ms: 1,
            duration_ms: 1200,
            character_count: 5,
            postprocess_rules_applied: 1,
            audio_quality: Some(AudioQualitySummary {
                rms: 0.01,
                peak: 0.5,
                silence_ratio: 0.7,
                active_speech_ms: 300,
                warnings: vec![AudioQualityWarning::LowVolume],
            }),
            audio_preprocess: None,
        }
    }

    #[test]
    fn saves_and_loads_audio_preprocess_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let mut next = entry("with-preprocess");
        next.audio_preprocess = Some(AudioPreprocessSummary {
            applied: true,
            original_sample_count: 32_000,
            processed_sample_count: 28_000,
            trimmed_front_samples: 2_000,
            trimmed_back_samples: 2_000,
            gain_applied: 2.0,
            fallback_to_raw: true,
        });

        save_transcript_history_entry(dir.path().to_path_buf(), next).unwrap();
        let loaded = load_transcript_history(dir.path().to_path_buf()).unwrap();

        assert_eq!(
            loaded[0].audio_preprocess.as_ref().unwrap().fallback_to_raw,
            true
        );
        assert_eq!(
            loaded[0].audio_preprocess.as_ref().unwrap().gain_applied,
            2.0
        );
    }

    #[test]
    fn saves_loads_deletes_and_clears_entries() {
        let dir = tempfile::tempdir().unwrap();
        let saved = save_transcript_history_entry(dir.path().to_path_buf(), entry("a")).unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(
            load_transcript_history(dir.path().to_path_buf()).unwrap()[0].id,
            "a"
        );
        assert!(
            load_transcript_history(dir.path().to_path_buf()).unwrap()[0]
                .audio_quality
                .is_some()
        );
        assert!(
            delete_transcript_history_entry(dir.path().to_path_buf(), "a".to_string())
                .unwrap()
                .is_empty()
        );
        assert!(clear_transcript_history(dir.path().to_path_buf())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn keeps_latest_500_entries() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..505 {
            save_transcript_history_entry(dir.path().to_path_buf(), entry(&format!("id-{i}")))
                .unwrap();
        }
        let loaded = load_transcript_history(dir.path().to_path_buf()).unwrap();
        assert_eq!(loaded.len(), 500);
        assert_eq!(loaded[0].id, "id-504");
        assert_eq!(loaded[499].id, "id-5");
    }

    #[test]
    fn corrupt_file_falls_back_to_empty_history() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path()).unwrap();
        std::fs::write(dir.path().join("transcript-history.json"), "not json").unwrap();
        assert!(load_transcript_history(dir.path().to_path_buf())
            .unwrap()
            .is_empty());
    }
}

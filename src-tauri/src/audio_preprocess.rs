use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::VoxError;

const AUDIO_PREPROCESS_CONFIG_FILE: &str = "audio-preprocess.json";

const HIGH_PASS_CUTOFF_HZ: f32 = 80.0;
const NORMALIZE_TARGET_RMS: f32 = 0.10;
const NORMALIZE_MAX_GAIN: f32 = 8.0;
const VAD_FRAME_MS: u32 = 20;
const VAD_PADDING_MS: u32 = 120;
const VAD_SPEECH_RMS: f32 = 0.01;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioPreprocessConfig {
    pub enabled: bool,
    pub remove_dc_offset: bool,
    pub high_pass_enabled: bool,
    pub normalize_enabled: bool,
    pub vad_trim_enabled: bool,
    pub denoise_enabled: bool,
}

impl Default for AudioPreprocessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            remove_dc_offset: true,
            high_pass_enabled: false,
            normalize_enabled: true,
            vad_trim_enabled: true,
            denoise_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioPreprocessSummary {
    pub applied: bool,
    pub original_sample_count: usize,
    pub processed_sample_count: usize,
    pub trimmed_front_samples: usize,
    pub trimmed_back_samples: usize,
    pub gain_applied: f32,
    #[serde(default)]
    pub fallback_to_raw: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioPreprocessResult {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub summary: AudioPreprocessSummary,
}

pub fn preprocess_asr_samples(
    samples: &[f32],
    sample_rate: u32,
    config: &AudioPreprocessConfig,
) -> AudioPreprocessResult {
    let original_sample_count = samples.len();
    if !config.enabled {
        return AudioPreprocessResult {
            samples: samples.to_vec(),
            sample_rate,
            summary: AudioPreprocessSummary {
                applied: false,
                original_sample_count,
                processed_sample_count: original_sample_count,
                trimmed_front_samples: 0,
                trimmed_back_samples: 0,
                gain_applied: 1.0,
                fallback_to_raw: false,
            },
        };
    }

    let mut processed = samples.to_vec();
    let mut applied = true;
    let mut trimmed_front_samples = 0;
    let mut trimmed_back_samples = 0;
    let mut gain_applied = 1.0;
    if config.remove_dc_offset {
        remove_dc_offset(&mut processed);
        applied = true;
    }
    if config.high_pass_enabled {
        apply_high_pass_filter(&mut processed, sample_rate);
        applied = true;
    }
    if config.normalize_enabled {
        let gain = normalize_rms(&mut processed);
        if gain > 1.0 {
            applied = true;
            gain_applied = gain;
        }
    }
    if config.vad_trim_enabled {
        let trimmed = trim_leading_trailing_silence(&processed, sample_rate);
        applied |= trimmed.trimmed;
        trimmed_front_samples = trimmed.trimmed_front_samples;
        trimmed_back_samples = trimmed.trimmed_back_samples;
        processed = trimmed.samples;
    }

    let processed_sample_count = processed.len();
    AudioPreprocessResult {
        samples: processed,
        sample_rate,
        summary: AudioPreprocessSummary {
            applied,
            original_sample_count,
            processed_sample_count,
            trimmed_front_samples,
            trimmed_back_samples,
            gain_applied,
            fallback_to_raw: false,
        },
    }
}

pub fn preprocess_i16_asr_samples(
    samples: &[i16],
    sample_rate: u32,
    config: &AudioPreprocessConfig,
) -> (Vec<i16>, AudioPreprocessSummary) {
    let normalized: Vec<f32> = samples
        .iter()
        .map(|sample| *sample as f32 / i16::MAX as f32)
        .collect();
    let result = preprocess_asr_samples(&normalized, sample_rate, config);
    let processed = result
        .samples
        .iter()
        .map(|sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16)
        .collect();
    (processed, result.summary)
}

pub fn remove_dc_offset(samples: &mut [f32]) {
    if samples.is_empty() {
        return;
    }
    let mean = samples.iter().sum::<f32>() / samples.len() as f32;
    for sample in samples {
        *sample -= mean;
    }
}

pub fn apply_high_pass_filter(samples: &mut [f32], sample_rate: u32) {
    if samples.len() < 2 || sample_rate == 0 {
        return;
    }
    let rc = 1.0 / (2.0 * std::f32::consts::PI * HIGH_PASS_CUTOFF_HZ);
    let dt = 1.0 / sample_rate as f32;
    let alpha = rc / (rc + dt);
    let mut previous_input = samples[0];
    let mut previous_output = samples[0];
    for sample in samples.iter_mut().skip(1) {
        let current_input = *sample;
        let current_output = alpha * (previous_output + current_input - previous_input);
        *sample = current_output;
        previous_input = current_input;
        previous_output = current_output;
    }
    samples[0] = 0.0;
}

pub fn normalize_rms(samples: &mut [f32]) -> f32 {
    if samples.is_empty() {
        return 1.0;
    }
    let rms = calculate_rms(samples);
    let peak = samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f32::max);
    if rms <= f32::EPSILON || peak >= 0.85 || rms >= NORMALIZE_TARGET_RMS {
        return 1.0;
    }
    let target_gain = NORMALIZE_TARGET_RMS / rms;
    let headroom_gain = if peak > f32::EPSILON {
        0.98 / peak
    } else {
        NORMALIZE_MAX_GAIN
    };
    let gain = target_gain.min(NORMALIZE_MAX_GAIN).min(headroom_gain);
    if gain <= 1.0 {
        return 1.0;
    }
    for sample in samples {
        *sample = (*sample * gain).clamp(-1.0, 1.0);
    }
    gain
}

#[derive(Debug, Clone, PartialEq)]
pub struct VadTrimResult {
    pub samples: Vec<f32>,
    pub trimmed: bool,
    pub trimmed_front_samples: usize,
    pub trimmed_back_samples: usize,
}

pub fn trim_leading_trailing_silence(samples: &[f32], sample_rate: u32) -> VadTrimResult {
    if samples.is_empty() || sample_rate == 0 {
        return VadTrimResult {
            samples: samples.to_vec(),
            trimmed: false,
            trimmed_front_samples: 0,
            trimmed_back_samples: 0,
        };
    }

    let frame_size = ((sample_rate as f32 * VAD_FRAME_MS as f32 / 1000.0).round() as usize).max(1);
    let speech_frames: Vec<usize> = samples
        .chunks(frame_size)
        .enumerate()
        .filter_map(|(index, frame)| (calculate_rms(frame) >= VAD_SPEECH_RMS).then_some(index))
        .collect();

    let Some(first_speech_frame) = speech_frames.first().copied() else {
        return VadTrimResult {
            samples: samples.to_vec(),
            trimmed: false,
            trimmed_front_samples: 0,
            trimmed_back_samples: 0,
        };
    };
    let last_speech_frame = speech_frames.last().copied().unwrap_or(first_speech_frame);
    let padding = (sample_rate as u64 * VAD_PADDING_MS as u64 / 1000) as usize;
    let start = first_speech_frame
        .saturating_mul(frame_size)
        .saturating_sub(padding);
    let speech_end = ((last_speech_frame + 1) * frame_size).min(samples.len());
    let end = speech_end.saturating_add(padding).min(samples.len());

    if start == 0 && end == samples.len() {
        return VadTrimResult {
            samples: samples.to_vec(),
            trimmed: false,
            trimmed_front_samples: 0,
            trimmed_back_samples: 0,
        };
    }

    VadTrimResult {
        samples: samples[start..end].to_vec(),
        trimmed: true,
        trimmed_front_samples: start,
        trimmed_back_samples: samples.len().saturating_sub(end),
    }
}

fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
}

pub fn load_audio_preprocess_config(config_dir: PathBuf) -> AudioPreprocessConfig {
    let path = audio_preprocess_config_path(&config_dir);
    let Ok(text) = std::fs::read_to_string(path) else {
        return AudioPreprocessConfig::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save_audio_preprocess_config(
    config_dir: PathBuf,
    config: AudioPreprocessConfig,
) -> Result<AudioPreprocessConfig, VoxError> {
    std::fs::create_dir_all(&config_dir).map_err(|error| {
        VoxError::Config(format!(
            "failed to create audio preprocess config directory: {error}"
        ))
    })?;
    let text = serde_json::to_string_pretty(&config).map_err(|error| {
        VoxError::Config(format!(
            "failed to serialize audio preprocess config: {error}"
        ))
    })?;
    std::fs::write(audio_preprocess_config_path(&config_dir), text).map_err(|error| {
        VoxError::Config(format!("failed to write audio preprocess config: {error}"))
    })?;
    Ok(config)
}

fn audio_preprocess_config_path(config_dir: &Path) -> PathBuf {
    config_dir.join(AUDIO_PREPROCESS_CONFIG_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mean(samples: &[f32]) -> f32 {
        samples.iter().sum::<f32>() / samples.len() as f32
    }

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
    }

    fn sine_wave(freq_hz: f32, seconds: f32, sample_rate: u32, amplitude: f32) -> Vec<f32> {
        let count = (seconds * sample_rate as f32) as usize;
        (0..count)
            .map(|index| {
                let t = index as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * freq_hz * t).sin() * amplitude
            })
            .collect()
    }

    #[test]
    fn high_pass_filter_reduces_low_frequency_more_than_speech_band() {
        let mut rumble = sine_wave(30.0, 0.5, 16_000, 0.5);
        let mut speech_band = sine_wave(300.0, 0.5, 16_000, 0.5);
        let rumble_before = rms(&rumble);
        let speech_before = rms(&speech_band);

        apply_high_pass_filter(&mut rumble, 16_000);
        apply_high_pass_filter(&mut speech_band, 16_000);

        let rumble_ratio = rms(&rumble) / rumble_before;
        let speech_ratio = rms(&speech_band) / speech_before;
        assert!(rumble_ratio < 0.45, "rumble ratio was {rumble_ratio}");
        assert!(speech_ratio > 0.80, "speech ratio was {speech_ratio}");
    }

    #[test]
    fn normalization_boosts_quiet_audio_without_clipping() {
        let mut samples = vec![0.01, -0.01, 0.012, -0.012];

        let gain = normalize_rms(&mut samples);

        assert!(gain > 1.0);
        assert!(rms(&samples) > 0.07);
        assert!(samples.iter().all(|sample| sample.abs() <= 1.0));
    }

    #[test]
    fn normalization_does_not_boost_near_clipping_audio() {
        let mut samples = vec![0.90, -0.92, 0.88, -0.91];
        let before = samples.clone();

        let gain = normalize_rms(&mut samples);

        assert_eq!(gain, 1.0);
        assert_eq!(samples, before);
    }

    #[test]
    fn vad_trim_removes_leading_and_trailing_silence_with_padding() {
        let sample_rate = 16_000;
        let mut samples = vec![0.0; 8_000];
        samples.extend((0..8_000).map(|index| if index % 2 == 0 { 0.08 } else { -0.08 }));
        samples.extend(vec![0.0; 8_000]);

        let result = trim_leading_trailing_silence(&samples, sample_rate);

        assert!(result.trimmed);
        assert!(result.samples.len() < samples.len());
        assert!(result.samples.len() >= 8_000 + 3_600);
        assert!(result.trimmed_front_samples >= 5_000);
        assert!(result.trimmed_back_samples >= 5_000);
    }

    #[test]
    fn vad_trim_returns_original_for_all_silence() {
        let samples = vec![0.0; 16_000];

        let result = trim_leading_trailing_silence(&samples, 16_000);

        assert!(!result.trimmed);
        assert_eq!(result.samples, samples);
    }

    #[test]
    fn saves_and_loads_audio_preprocess_config() {
        let dir = tempfile::tempdir().unwrap();
        let config = AudioPreprocessConfig {
            enabled: true,
            high_pass_enabled: true,
            ..AudioPreprocessConfig::default()
        };

        save_audio_preprocess_config(dir.path().to_path_buf(), config.clone()).unwrap();
        let loaded = load_audio_preprocess_config(dir.path().to_path_buf());

        assert_eq!(loaded, config);
    }

    #[test]
    fn missing_audio_preprocess_config_uses_default() {
        let dir = tempfile::tempdir().unwrap();

        let loaded = load_audio_preprocess_config(dir.path().to_path_buf());

        assert_eq!(loaded, AudioPreprocessConfig::default());
    }

    #[test]
    fn disabled_i16_pipeline_returns_samples_unchanged() {
        let samples = vec![100_i16, -200, 300, -400];

        let (processed, summary) =
            preprocess_i16_asr_samples(&samples, 16_000, &AudioPreprocessConfig::default());

        assert_eq!(processed, samples);
        assert!(!summary.applied);
    }

    #[test]
    fn default_config_keeps_pipeline_disabled() {
        let config = AudioPreprocessConfig::default();

        assert!(!config.enabled);
        assert!(config.remove_dc_offset);
        assert!(!config.high_pass_enabled);
        assert!(config.normalize_enabled);
        assert!(config.vad_trim_enabled);
        assert!(!config.denoise_enabled);
    }

    #[test]
    fn disabled_pipeline_returns_samples_unchanged() {
        let config = AudioPreprocessConfig::default();
        let samples = vec![0.10, -0.20, 0.30, -0.40];

        let result = preprocess_asr_samples(&samples, 16_000, &config);

        assert_eq!(result.samples, samples);
        assert!(!result.summary.applied);
    }

    #[test]
    fn enabled_pipeline_reports_processed_even_without_material_change() {
        let config = AudioPreprocessConfig {
            enabled: true,
            remove_dc_offset: false,
            high_pass_enabled: false,
            normalize_enabled: true,
            vad_trim_enabled: true,
            denoise_enabled: false,
        };
        let samples = vec![0.12, -0.12, 0.12, -0.12];

        let result = preprocess_asr_samples(&samples, 16_000, &config);

        assert!(result.summary.applied);
        assert_eq!(result.summary.gain_applied, 1.0);
        assert_eq!(result.summary.trimmed_front_samples, 0);
        assert_eq!(result.summary.trimmed_back_samples, 0);
        assert_eq!(result.samples, samples);
    }

    #[test]
    fn enabled_pipeline_applies_dc_offset_removal() {
        let config = AudioPreprocessConfig {
            enabled: true,
            ..AudioPreprocessConfig::default()
        };
        let samples = vec![0.25, 0.30, 0.35, 0.40];

        let result = preprocess_asr_samples(&samples, 16_000, &config);

        assert!(result.summary.applied);
        assert!(mean(&result.samples).abs() < 0.000001);
        assert_eq!(result.sample_rate, 16_000);
    }

    #[test]
    fn remove_dc_offset_allows_empty_samples() {
        let mut samples = Vec::new();

        remove_dc_offset(&mut samples);

        assert!(samples.is_empty());
    }

    #[test]
    fn remove_dc_offset_centers_samples_around_zero() {
        let mut samples = vec![0.25, 0.30, 0.35, 0.40];

        remove_dc_offset(&mut samples);

        assert!(mean(&samples).abs() < 0.000001);
        assert_eq!(samples.len(), 4);
    }
}

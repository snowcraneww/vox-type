use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AudioQualityWarning {
    LowVolume,
    ClippingRisk,
    MostlySilence,
    PossibleFarMicrophone,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioQualitySummary {
    pub rms: f64,
    pub peak: f64,
    pub silence_ratio: f64,
    pub active_speech_ms: u64,
    pub warnings: Vec<AudioQualityWarning>,
}

const LOW_VOLUME_RMS: f64 = 0.004;
const LOW_VOLUME_PEAK: f64 = 0.018;
const FAR_MIC_RMS: f64 = 0.006;
const CLIPPING_PEAK: f64 = 0.999;
const SILENCE_FRAME_RMS: f64 = 0.004;
const MOSTLY_SILENCE_RATIO: f64 = 0.995;
const MOSTLY_SILENCE_MAX_ACTIVE_MS: u64 = 80;
const FAR_MIC_SILENCE_RATIO: f64 = 0.90;
const FAR_MIC_PEAK: f64 = 0.025;
const FAR_MIC_MIN_ACTIVE_MS: u64 = 250;

pub fn analyze_audio_quality(samples: &[i16], sample_rate: u32) -> AudioQualitySummary {
    if samples.is_empty() || sample_rate == 0 {
        return AudioQualitySummary {
            rms: 0.0,
            peak: 0.0,
            silence_ratio: 1.0,
            active_speech_ms: 0,
            warnings: vec![
                AudioQualityWarning::LowVolume,
                AudioQualityWarning::MostlySilence,
            ],
        };
    }

    let normalized: Vec<f64> = samples
        .iter()
        .map(|sample| *sample as f64 / i16::MAX as f64)
        .collect();
    let peak = normalized
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max);
    let rms = (normalized.iter().map(|sample| sample * sample).sum::<f64>()
        / normalized.len() as f64)
        .sqrt();

    let frame_size = ((sample_rate as f64 * 0.02).round() as usize).max(1);
    let mut frame_count = 0usize;
    let mut silent_frames = 0usize;
    for frame in normalized.chunks(frame_size) {
        frame_count += 1;
        let frame_rms =
            (frame.iter().map(|sample| sample * sample).sum::<f64>() / frame.len() as f64).sqrt();
        if frame_rms < SILENCE_FRAME_RMS {
            silent_frames += 1;
        }
    }
    let silence_ratio = if frame_count == 0 {
        1.0
    } else {
        silent_frames as f64 / frame_count as f64
    };
    let active_frames = frame_count.saturating_sub(silent_frames);
    let active_speech_ms =
        ((active_frames * frame_size) as f64 / sample_rate as f64 * 1000.0).round() as u64;

    let mut warnings = Vec::new();
    if rms < LOW_VOLUME_RMS && peak < LOW_VOLUME_PEAK {
        warnings.push(AudioQualityWarning::LowVolume);
    }
    if peak > CLIPPING_PEAK {
        warnings.push(AudioQualityWarning::ClippingRisk);
    }
    if silence_ratio > MOSTLY_SILENCE_RATIO && active_speech_ms <= MOSTLY_SILENCE_MAX_ACTIVE_MS {
        warnings.push(AudioQualityWarning::MostlySilence);
    }
    if rms < FAR_MIC_RMS
        && peak < FAR_MIC_PEAK
        && silence_ratio > FAR_MIC_SILENCE_RATIO
        && active_speech_ms >= FAR_MIC_MIN_ACTIVE_MS
    {
        warnings.push(AudioQualityWarning::PossibleFarMicrophone);
    }

    AudioQualitySummary {
        rms,
        peak,
        silence_ratio,
        active_speech_ms,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_zero_samples_are_low_volume_and_mostly_silence() {
        let summary = analyze_audio_quality(&vec![0; 16000], 16000);
        assert!(summary.warnings.contains(&AudioQualityWarning::LowVolume));
        assert!(summary
            .warnings
            .contains(&AudioQualityWarning::MostlySilence));
        assert_eq!(summary.active_speech_ms, 0);
    }

    #[test]
    fn near_full_scale_samples_report_clipping_risk() {
        let summary = analyze_audio_quality(&vec![32760; 16000], 16000);
        assert!(summary
            .warnings
            .contains(&AudioQualityWarning::ClippingRisk));
    }

    #[test]
    fn very_low_level_sparse_speech_reports_far_microphone() {
        let mut samples = vec![0; 64000];
        for (index, sample) in samples.iter_mut().take(4800).enumerate() {
            *sample = if index % 2 == 0 { 80 } else { -80 };
        }
        let summary = analyze_audio_quality(&samples, 16000);
        assert!(summary.silence_ratio > 0.90);
        assert!(summary.active_speech_ms >= 250);
        assert!(summary
            .warnings
            .contains(&AudioQualityWarning::PossibleFarMicrophone));
    }

    #[test]
    fn moderate_low_level_speech_does_not_warn() {
        let samples: Vec<i16> = (0..16000)
            .map(|i| if i % 2 == 0 { 1400 } else { -1400 })
            .collect();
        let summary = analyze_audio_quality(&samples, 16000);
        assert!(summary.warnings.is_empty());
    }

    #[test]
    fn mostly_silence_requires_almost_no_active_speech() {
        let mut samples = vec![0; 32000];
        for sample in samples.iter_mut().take(2400) {
            *sample = 3500;
        }
        let summary = analyze_audio_quality(&samples, 16000);
        assert!(!summary
            .warnings
            .contains(&AudioQualityWarning::MostlySilence));
    }

    #[test]
    fn almost_empty_recording_reports_mostly_silence() {
        let mut samples = vec![0; 80000];
        for sample in samples.iter_mut().take(320) {
            *sample = 3500;
        }
        let summary = analyze_audio_quality(&samples, 16000);
        assert!(summary.silence_ratio > 0.995);
        assert!(summary.active_speech_ms <= 80);
        assert!(summary
            .warnings
            .contains(&AudioQualityWarning::MostlySilence));
    }

    #[test]
    fn high_but_not_full_scale_samples_do_not_report_clipping() {
        let summary = analyze_audio_quality(&vec![32000; 16000], 16000);
        assert!(!summary
            .warnings
            .contains(&AudioQualityWarning::ClippingRisk));
    }

    #[test]
    fn sparse_but_present_speech_does_not_report_mostly_silence() {
        let mut samples = vec![0; 48000];
        for sample in samples.iter_mut().take(16000) {
            *sample = 3000;
        }
        let summary = analyze_audio_quality(&samples, 16000);
        assert!(summary.silence_ratio > 0.60);
        assert!(summary.active_speech_ms >= 1000);
        assert!(!summary
            .warnings
            .contains(&AudioQualityWarning::MostlySilence));
    }

    #[test]
    fn quiet_continuous_speech_does_not_report_far_microphone_by_itself() {
        let samples: Vec<i16> = (0..16000)
            .map(|i| if i % 2 == 0 { 1000 } else { -1000 })
            .collect();
        let summary = analyze_audio_quality(&samples, 16000);
        assert!(!summary
            .warnings
            .contains(&AudioQualityWarning::PossibleFarMicrophone));
    }

    #[test]
    fn normal_speech_like_samples_have_no_warning() {
        let samples: Vec<i16> = (0..16000)
            .map(|i| if i % 2 == 0 { 5000 } else { -5000 })
            .collect();
        let summary = analyze_audio_quality(&samples, 16000);
        assert!(summary.warnings.is_empty());
    }
}

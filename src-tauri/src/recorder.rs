use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};

use crate::audio;
use crate::error::VoxError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RecorderState {
    #[default]
    Idle,
    Recording,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordingBuffer {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u16,
    source_channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecordedAudio {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_count: usize,
    pub duration_ms: u64,
    pub peak_amplitude: i16,
    pub rms_amplitude: u16,
    pub asr_samples: Vec<i16>,
    pub asr_sample_rate: u32,
    pub asr_sample_count: usize,
    pub asr_duration_ms: u64,
}

impl RecordedAudio {
    pub fn asr_samples_from_source_index(
        &self,
        from_sample_index: usize,
    ) -> Result<Vec<i16>, VoxError> {
        if from_sample_index > self.samples.len() {
            return Err(VoxError::Recorder(format!(
                "录音片段起点超出最近录音样本数：{from_sample_index} > {}",
                self.samples.len()
            )));
        }
        Ok(audio::resample_mono_i16(
            &self.samples[from_sample_index..],
            self.sample_rate,
            audio::TARGET_SAMPLE_RATE,
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordingSession {
    buffer: RecordingBuffer,
    state: RecorderState,
}

impl RecordingSession {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            buffer: RecordingBuffer::empty(sample_rate, channels),
            state: RecorderState::Recording,
        }
    }

    pub fn push_interleaved_i16(&mut self, input: &[i16]) -> Result<(), VoxError> {
        if self.state != RecorderState::Recording {
            return Err(VoxError::Recorder("录音已经停止".to_string()));
        }
        self.buffer.push_interleaved_i16(input);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<RecordedAudio, VoxError> {
        if self.state != RecorderState::Recording {
            return Err(VoxError::Recorder("录音已经停止".to_string()));
        }
        self.state = RecorderState::Idle;
        let sample_count = self.buffer.samples.len();
        let asr_samples = audio::resample_mono_i16(
            &self.buffer.samples,
            self.buffer.sample_rate,
            audio::TARGET_SAMPLE_RATE,
        );
        let asr_sample_count = asr_samples.len();
        let peak_amplitude = peak_amplitude(&self.buffer.samples);
        let rms_amplitude = rms_amplitude(&self.buffer.samples);
        Ok(RecordedAudio {
            samples: self.buffer.samples.clone(),
            sample_rate: self.buffer.sample_rate,
            channels: self.buffer.channels,
            sample_count,
            duration_ms: audio::duration_ms(sample_count, self.buffer.sample_rate),
            peak_amplitude,
            rms_amplitude,
            asr_samples,
            asr_sample_rate: audio::TARGET_SAMPLE_RATE,
            asr_sample_count,
            asr_duration_ms: audio::duration_ms(asr_sample_count, audio::TARGET_SAMPLE_RATE),
        })
    }

    pub fn sample_count(&self) -> usize {
        self.buffer.samples.len()
    }

    pub fn asr_samples_from(&self, from_sample_index: usize) -> Result<Vec<i16>, VoxError> {
        if from_sample_index > self.buffer.samples.len() {
            return Err(VoxError::Recorder(format!(
                "录音片段起点超出当前样本数：{from_sample_index} > {}",
                self.buffer.samples.len()
            )));
        }
        let source_samples = &self.buffer.samples[from_sample_index..];
        Ok(audio::resample_mono_i16(
            source_samples,
            self.buffer.sample_rate,
            audio::TARGET_SAMPLE_RATE,
        ))
    }

    pub fn source_sample_count(&self) -> usize {
        self.buffer.samples.len()
    }
}

fn peak_amplitude(samples: &[i16]) -> i16 {
    samples
        .iter()
        .map(|sample| sample.saturating_abs())
        .max()
        .unwrap_or(0)
}

fn rms_amplitude(samples: &[i16]) -> u16 {
    if samples.is_empty() {
        return 0;
    }
    let sum_squares: f64 = samples
        .iter()
        .map(|sample| {
            let value = *sample as f64;
            value * value
        })
        .sum();
    (sum_squares / samples.len() as f64).sqrt().round() as u16
}

impl RecordingBuffer {
    pub fn empty(sample_rate: u32, channels: u16) -> Self {
        Self {
            samples: Vec::new(),
            sample_rate,
            channels,
            source_channels: channels,
        }
    }

    pub fn push_interleaved_i16(&mut self, input: &[i16]) {
        if self.source_channels <= 1 {
            self.samples.extend_from_slice(input);
            self.channels = 1;
            return;
        }

        for frame in input.chunks(self.source_channels as usize) {
            let sum: i32 = frame.iter().map(|sample| *sample as i32).sum();
            self.samples.push((sum / frame.len() as i32) as i16);
        }
        self.channels = 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecorderInfo {
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecorderRuntimeStatus {
    pub state: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub sample_count: usize,
    pub duration_ms: u64,
}

#[derive(Default)]
pub struct RecorderManager {
    active: Mutex<Option<ActiveRecording>>,
    last_recording: Mutex<Option<RecordedAudio>>,
    selected_device_name: Mutex<Option<String>>,
}

struct ActiveRecording {
    session: Arc<Mutex<RecordingSession>>,
    stream: cpal::Stream,
    sample_rate: u32,
    channels: u16,
}

impl RecorderManager {
    pub fn start(&self) -> Result<RecorderRuntimeStatus, VoxError> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| VoxError::Recorder("录音状态锁已损坏".to_string()))?;
        if active.is_some() {
            return Err(VoxError::Recorder("录音已经在进行中".to_string()));
        }

        let host = cpal::default_host();
        let selected_device_name = self
            .selected_device_name
            .lock()
            .map_err(|_| VoxError::Recorder("输入设备选择锁已损坏".to_string()))?
            .clone();
        let device = input_device_by_name(&host, selected_device_name.as_deref())?;
        let config = device
            .default_input_config()
            .map_err(|error| VoxError::Recorder(format!("读取默认输入配置失败：{error}")))?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let session = Arc::new(Mutex::new(RecordingSession::new(sample_rate, channels)));
        let stream = build_input_stream(&device, &config, Arc::clone(&session))?;
        stream
            .play()
            .map_err(|error| VoxError::Recorder(format!("启动录音流失败：{error}")))?;

        *active = Some(ActiveRecording {
            session,
            stream,
            sample_rate,
            channels,
        });

        Ok(RecorderRuntimeStatus {
            state: "recording".to_string(),
            sample_rate: Some(sample_rate),
            channels: Some(channels),
            sample_count: 0,
            duration_ms: 0,
        })
    }

    pub fn stop(&self) -> Result<RecordedAudio, VoxError> {
        let active = self
            .active
            .lock()
            .map_err(|_| VoxError::Recorder("录音状态锁已损坏".to_string()))?
            .take()
            .ok_or_else(|| VoxError::Recorder("没有正在进行的录音".to_string()))?;
        drop(active.stream);
        let mut session = active
            .session
            .lock()
            .map_err(|_| VoxError::Recorder("录音缓冲区锁已损坏".to_string()))?;
        let recorded = session.stop()?;
        *self
            .last_recording
            .lock()
            .map_err(|_| VoxError::Recorder("最后录音锁已损坏".to_string()))? =
            Some(recorded.clone());
        Ok(recorded)
    }

    pub fn status(&self) -> Result<RecorderRuntimeStatus, VoxError> {
        let active = self
            .active
            .lock()
            .map_err(|_| VoxError::Recorder("录音状态锁已损坏".to_string()))?;
        let Some(active) = active.as_ref() else {
            return Ok(RecorderRuntimeStatus {
                state: "idle".to_string(),
                sample_rate: None,
                channels: None,
                sample_count: 0,
                duration_ms: 0,
            });
        };
        let session = active
            .session
            .lock()
            .map_err(|_| VoxError::Recorder("录音缓冲区锁已损坏".to_string()))?;
        let sample_count = session.sample_count();
        Ok(RecorderRuntimeStatus {
            state: "recording".to_string(),
            sample_rate: Some(active.sample_rate),
            channels: Some(active.channels),
            sample_count,
            duration_ms: audio::duration_ms(sample_count, active.sample_rate),
        })
    }

    pub fn last_asr_samples(&self) -> Result<Vec<i16>, VoxError> {
        let last_recording = self
            .last_recording
            .lock()
            .map_err(|_| VoxError::Recorder("最后录音锁已损坏".to_string()))?;
        let recording = last_recording
            .as_ref()
            .ok_or_else(|| VoxError::Recorder("还没有可转写的录音".to_string()))?;
        if recording.asr_samples.is_empty() {
            return Err(VoxError::Recorder("最后一次录音没有 ASR 样本".to_string()));
        }
        Ok(recording.asr_samples.clone())
    }

    pub fn last_recording(&self) -> Result<RecordedAudio, VoxError> {
        let last_recording = self
            .last_recording
            .lock()
            .map_err(|_| VoxError::Recorder("最后录音锁已损坏".to_string()))?;
        last_recording
            .clone()
            .ok_or_else(|| VoxError::Recorder("还没有可导出的录音".to_string()))
    }

    pub fn last_asr_samples_from(
        &self,
        from_sample_index: usize,
    ) -> Result<(Vec<i16>, usize), VoxError> {
        let recording = self.last_recording()?;
        let end_sample_index = recording.samples.len();
        Ok((
            recording.asr_samples_from_source_index(from_sample_index)?,
            end_sample_index,
        ))
    }

    pub fn active_asr_samples_from(
        &self,
        from_sample_index: usize,
    ) -> Result<(Vec<i16>, usize), VoxError> {
        let active = self
            .active
            .lock()
            .map_err(|_| VoxError::Recorder("录音状态锁已损坏".to_string()))?;
        let active = active
            .as_ref()
            .ok_or_else(|| VoxError::Recorder("没有正在进行的录音".to_string()))?;
        let session = active
            .session
            .lock()
            .map_err(|_| VoxError::Recorder("录音缓冲区锁已损坏".to_string()))?;
        let end_sample_index = session.source_sample_count();
        Ok((
            session.asr_samples_from(from_sample_index)?,
            end_sample_index,
        ))
    }

    pub fn set_input_device(&self, device_name: Option<String>) -> Result<RecorderInfo, VoxError> {
        if self
            .active
            .lock()
            .map_err(|_| VoxError::Recorder("录音状态锁已损坏".to_string()))?
            .is_some()
        {
            return Err(VoxError::Recorder("录音进行中不能切换输入设备".to_string()));
        }

        let normalized = device_name.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });
        let host = cpal::default_host();
        let device = input_device_by_name(&host, normalized.as_deref())?;
        let info = input_info_from_device(&device)?;
        *self
            .selected_device_name
            .lock()
            .map_err(|_| VoxError::Recorder("输入设备选择锁已损坏".to_string()))? = normalized;
        Ok(info)
    }

    pub fn selected_device_name(&self) -> Result<Option<String>, VoxError> {
        self.selected_device_name
            .lock()
            .map_err(|_| VoxError::Recorder("输入设备选择锁已损坏".to_string()))
            .map(|value| value.clone())
    }
}

pub fn default_input_info() -> Result<RecorderInfo, VoxError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| VoxError::Recorder("没有找到默认输入设备".to_string()))?;
    input_info_from_device(&device)
}

pub fn list_input_devices() -> Result<Vec<RecorderInfo>, VoxError> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|error| VoxError::Recorder(format!("读取输入设备列表失败：{error}")))?;
    let mut infos = Vec::new();
    for device in devices {
        if let Ok(info) = input_info_from_device(&device) {
            infos.push(info);
        }
    }
    if infos.is_empty() {
        return Err(VoxError::Recorder("没有找到可用输入设备".to_string()));
    }
    Ok(infos)
}

fn input_device_by_name(
    host: &cpal::Host,
    selected_device_name: Option<&str>,
) -> Result<cpal::Device, VoxError> {
    if let Some(selected_device_name) = selected_device_name {
        let devices = host
            .input_devices()
            .map_err(|error| VoxError::Recorder(format!("读取输入设备列表失败：{error}")))?;
        for device in devices {
            let name = device.name().unwrap_or_default();
            if name == selected_device_name {
                return Ok(device);
            }
        }
        return Err(VoxError::Recorder(format!(
            "没有找到已选择的输入设备：{selected_device_name}"
        )));
    }

    host.default_input_device()
        .ok_or_else(|| VoxError::Recorder("没有找到默认输入设备".to_string()))
}

fn input_info_from_device(device: &cpal::Device) -> Result<RecorderInfo, VoxError> {
    let config = device
        .default_input_config()
        .map_err(|error| VoxError::Recorder(format!("读取输入配置失败：{error}")))?;
    let device_name = device.name().unwrap_or_else(|_| "未知麦克风".to_string());

    Ok(RecorderInfo {
        device_name,
        sample_rate: config.sample_rate().0,
        channels: config.channels(),
    })
}

pub fn normalize_to_mono_i16(input: &[i16], channels: u16, sample_rate: u32) -> RecordingBuffer {
    let mut buffer = RecordingBuffer::empty(sample_rate, channels);
    buffer.push_interleaved_i16(input);
    buffer
}

fn build_input_stream(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    session: Arc<Mutex<RecordingSession>>,
) -> Result<cpal::Stream, VoxError> {
    let stream_config = config.config();
    let error_callback = |error| eprintln!("VoxType 录音流错误：{error}");
    match config.sample_format() {
        cpal::SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _| push_input_samples(&session, data),
            error_callback,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &stream_config,
            move |data: &[u16], _| {
                let converted: Vec<i16> = data
                    .iter()
                    .map(|sample| (*sample as i32 - 32_768) as i16)
                    .collect();
                push_input_samples(&session, &converted);
            },
            error_callback,
            None,
        ),
        cpal::SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| {
                let converted: Vec<i16> = data
                    .iter()
                    .map(|sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                    .collect();
                push_input_samples(&session, &converted);
            },
            error_callback,
            None,
        ),
        sample_format => {
            return Err(VoxError::Recorder(format!(
                "暂不支持的输入采样格式：{sample_format:?}"
            )));
        }
    }
    .map_err(|error| VoxError::Recorder(format!("创建录音流失败：{error}")))
}

fn push_input_samples(session: &Arc<Mutex<RecordingSession>>, samples: &[i16]) {
    if let Ok(mut session) = session.lock() {
        let _ = session.push_interleaved_i16(samples);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_samples_are_mixed_to_mono() {
        let mut buffer = RecordingBuffer::empty(16_000, 2);
        buffer.push_interleaved_i16(&[100, 300, -100, -300]);
        assert_eq!(buffer.samples, vec![200, -200]);
        assert_eq!(buffer.channels, 1);
    }

    #[test]
    fn stereo_samples_remain_mixed_across_multiple_callbacks() {
        let mut buffer = RecordingBuffer::empty(16_000, 2);

        buffer.push_interleaved_i16(&[100, 300, -100, -300]);
        buffer.push_interleaved_i16(&[200, 400, -200, -400]);

        assert_eq!(buffer.samples, vec![200, -200, 300, -300]);
        assert_eq!(buffer.channels, 1);
    }

    #[test]
    fn normalize_keeps_mono_samples() {
        let buffer = normalize_to_mono_i16(&[1, -1, 2], 1, 16_000);
        assert_eq!(buffer.samples, vec![1, -1, 2]);
        assert_eq!(buffer.sample_rate, 16_000);
    }

    #[test]
    fn recording_session_collects_samples_and_returns_summary_on_stop() {
        let mut session = RecordingSession::new(16_000, 2);
        session
            .push_interleaved_i16(&[100, 300, -100, -300])
            .unwrap();

        let result = session.stop().unwrap();

        assert_eq!(result.sample_rate, 16_000);
        assert_eq!(result.channels, 1);
        assert_eq!(result.sample_count, 2);
        assert_eq!(result.duration_ms, 0);
        assert_eq!(result.samples, vec![200, -200]);
        assert_eq!(result.asr_sample_rate, 16_000);
        assert_eq!(result.asr_sample_count, 2);
    }

    #[test]
    fn stopped_recording_session_rejects_more_samples() {
        let mut session = RecordingSession::new(16_000, 1);
        let _ = session.stop().unwrap();

        let error = session.push_interleaved_i16(&[1, 2, 3]).unwrap_err();

        assert!(error.to_string().contains("录音已经停止"));
    }

    #[test]
    fn recorded_audio_exposes_asr_samples_at_target_rate() {
        let mut session = RecordingSession::new(48_000, 1);
        session.push_interleaved_i16(&[0, 1, 2, 3, 4, 5]).unwrap();

        let result = session.stop().unwrap();

        assert_eq!(result.asr_sample_rate, 16_000);
        assert_eq!(result.asr_samples, vec![0, 3]);
        assert_eq!(result.asr_sample_count, 2);
    }

    #[test]
    fn recorded_audio_exposes_volume_summary() {
        let mut session = RecordingSession::new(16_000, 1);
        session.push_interleaved_i16(&[0, 3000, -4000]).unwrap();

        let result = session.stop().unwrap();

        assert_eq!(result.peak_amplitude, 4000);
        assert!(result.rms_amplitude > 2800);
    }

    #[test]
    fn recorded_audio_returns_tail_asr_samples_from_source_index() {
        let mut session = RecordingSession::new(16_000, 1);
        session.push_interleaved_i16(&[0, 100, 200, 300]).unwrap();
        let result = session.stop().unwrap();

        let tail = result.asr_samples_from_source_index(2).unwrap();

        assert_eq!(tail, vec![200, 300]);
    }

    #[test]
    fn recorded_audio_rejects_tail_start_past_sample_count() {
        let mut session = RecordingSession::new(16_000, 1);
        session.push_interleaved_i16(&[0, 100]).unwrap();
        let result = session.stop().unwrap();

        let error = result.asr_samples_from_source_index(3).unwrap_err();

        assert!(error.to_string().contains("录音片段起点超出最近录音样本数"));
    }

    #[test]
    fn recording_session_returns_asr_delta_from_source_index() {
        let mut session = RecordingSession::new(16_000, 1);
        session.push_interleaved_i16(&[0, 100, 200, 300]).unwrap();

        let delta = session.asr_samples_from(2).unwrap();

        assert_eq!(delta, vec![200, 300]);
        assert_eq!(session.source_sample_count(), 4);
    }

    #[test]
    fn recording_session_rejects_delta_start_past_current_samples() {
        let mut session = RecordingSession::new(16_000, 1);
        session.push_interleaved_i16(&[0, 100]).unwrap();

        let error = session.asr_samples_from(3).unwrap_err();

        assert!(error.to_string().contains("录音片段起点超出当前样本数"));
    }

    #[test]
    fn recorder_manager_tracks_selected_device_name_without_hardware() {
        let manager = RecorderManager::default();
        assert_eq!(manager.selected_device_name().unwrap(), None);
    }
}

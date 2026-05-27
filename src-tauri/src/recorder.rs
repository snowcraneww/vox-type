use serde::{Deserialize, Serialize};

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
}

impl RecordingBuffer {
    pub fn empty(sample_rate: u32, channels: u16) -> Self {
        Self {
            samples: Vec::new(),
            sample_rate,
            channels,
        }
    }

    pub fn push_interleaved_i16(&mut self, input: &[i16]) {
        if self.channels <= 1 {
            self.samples.extend_from_slice(input);
            return;
        }

        for frame in input.chunks(self.channels as usize) {
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

pub fn default_input_info() -> Result<RecorderInfo, VoxError> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| VoxError::Recorder("没有找到默认输入设备".to_string()))?;
    let config = device
        .default_input_config()
        .map_err(|error| VoxError::Recorder(format!("读取默认输入配置失败：{error}")))?;
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
    fn normalize_keeps_mono_samples() {
        let buffer = normalize_to_mono_i16(&[1, -1, 2], 1, 16_000);
        assert_eq!(buffer.samples, vec![1, -1, 2]);
        assert_eq!(buffer.sample_rate, 16_000);
    }
}

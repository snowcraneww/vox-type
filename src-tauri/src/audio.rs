pub const TARGET_SAMPLE_RATE: u32 = 16_000;

pub fn duration_ms(sample_count: usize, sample_rate: u32) -> u64 {
    if sample_rate == 0 {
        return 0;
    }
    ((sample_count as u64) * 1000) / sample_rate as u64
}

pub fn is_too_short(sample_count: usize, sample_rate: u32, min_duration_ms: u64) -> bool {
    duration_ms(sample_count, sample_rate) < min_duration_ms
}

pub fn resample_mono_i16(input: &[i16], source_rate: u32, target_rate: u32) -> Vec<i16> {
    if input.is_empty() || source_rate == 0 || target_rate == 0 {
        return Vec::new();
    }
    if source_rate == target_rate {
        return input.to_vec();
    }

    let output_len = ((input.len() as u64 * target_rate as u64) / source_rate as u64) as usize;
    let output_len = output_len.max(1);
    (0..output_len)
        .map(|index| {
            let source_index = (index as u64 * source_rate as u64) / target_rate as u64;
            input[source_index.min(input.len() as u64 - 1) as usize]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_duration_ms() {
        assert_eq!(duration_ms(16_000, TARGET_SAMPLE_RATE), 1000);
    }

    #[test]
    fn detects_short_audio() {
        assert!(is_too_short(800, TARGET_SAMPLE_RATE, 100));
        assert!(!is_too_short(3_200, TARGET_SAMPLE_RATE, 100));
    }

    #[test]
    fn resamples_mono_pcm_down_to_target_rate() {
        let input = vec![0, 1, 2, 3, 4, 5];

        let output = resample_mono_i16(&input, 48_000, 16_000);

        assert_eq!(output, vec![0, 3]);
    }

    #[test]
    fn resampling_keeps_samples_when_rate_already_matches() {
        let input = vec![10, -10, 20];

        let output = resample_mono_i16(&input, TARGET_SAMPLE_RATE, TARGET_SAMPLE_RATE);

        assert_eq!(output, input);
    }
}

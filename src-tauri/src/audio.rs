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
}

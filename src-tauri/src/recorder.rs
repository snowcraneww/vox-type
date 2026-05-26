#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecorderState {
    Idle,
    Recording,
}

impl Default for RecorderState {
    fn default() -> Self {
        Self::Idle
    }
}

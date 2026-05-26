#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RecorderState {
    #[default]
    Idle,
    Recording,
}

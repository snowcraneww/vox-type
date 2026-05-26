#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub accelerator: String,
}

impl Default for HotkeyBinding {
    fn default() -> Self {
        Self {
            accelerator: "Ctrl+Alt+Space".to_string(),
        }
    }
}

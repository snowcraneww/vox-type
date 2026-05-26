use crate::error::VoxError;

pub trait InsertionStrategy {
    fn insert_text(&self, text: &str) -> Result<(), VoxError>;
}

#[derive(Debug, Default)]
pub struct MockInsertion;

impl InsertionStrategy for MockInsertion {
    fn insert_text(&self, text: &str) -> Result<(), VoxError> {
        if text.trim().is_empty() {
            return Err(VoxError::Insertion("不能上屏空文本".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ClipboardInsertion;

impl InsertionStrategy for ClipboardInsertion {
    fn insert_text(&self, _text: &str) -> Result<(), VoxError> {
        Err(VoxError::Insertion(
            "剪贴板上屏尚未接入 Windows 实现".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_insertion_rejects_empty_text() {
        let error = MockInsertion::default().insert_text(" ").unwrap_err();
        assert!(error.to_string().contains("不能上屏空文本"));
    }
}

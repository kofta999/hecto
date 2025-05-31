use super::filetype::FileType;
use crate::prelude::Location;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DocumentStatus {
    pub filename: String,
    pub line_count: usize,
    pub text_location: Location,
    pub is_modified: bool,
    pub file_type: FileType,
}

impl DocumentStatus {
    pub fn modified_indicator_to_string(&self) -> String {
        if self.is_modified {
            String::from("(modified)")
        } else {
            String::new()
        }
    }

    pub fn line_count_to_string(&self) -> String {
        format!("{} lines", self.line_count)
    }

    pub fn position_indicator_to_string(&self) -> String {
        format!(
            "{}:{}",
            self.text_location.line_idx, self.text_location.grapheme_idx,
        )
    }

    pub fn file_type_to_string(&self) -> String {
        self.file_type.to_string()
    }
}

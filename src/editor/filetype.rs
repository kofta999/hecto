use std::fmt::Display;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Rust,
    #[default]
    Text,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Rust => write!(f, "Rust"),
            FileType::Text => write!(f, "Text"),
        }
    }
}

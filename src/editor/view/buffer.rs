use super::line::Line;
use std::fs;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

/// Where the text resides
impl Buffer {
    /// Checks if a buffer is empty
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 0
    }

    /// Loads a file into a buffer
    pub fn load(filename: &str) -> Result<Self, std::io::Error> {
        let file_contents = fs::read_to_string(filename)?;
        let mut lines = Vec::new();

        for line in file_contents.lines() {
            lines.push(Line::from(line));
        }

        Ok(Self { lines })
    }

    /// Returns the length of buffer lines
    pub fn height(&self) -> usize {
        self.lines.len()
    }
}

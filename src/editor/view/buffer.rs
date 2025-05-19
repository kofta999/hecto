use super::line::Line;
use std::fs;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 0
    }

    pub fn load(filename: &str) -> Result<Self, std::io::Error> {
        let file_contents = fs::read_to_string(filename)?;
        let mut lines = Vec::new();

        for line in file_contents.lines() {
            lines.push(Line::from(line));
        }

        Ok(Self { lines })
    }
}

use std::fs;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 0
    }

    pub fn load(filename: &str) -> Result<Self, std::io::Error> {
        let file_contents = fs::read_to_string(filename)?;
        let mut lines = Vec::new();

        for line in file_contents.lines() {
            lines.push(line.to_string());
        }

        Ok(Self { lines })
    }
}

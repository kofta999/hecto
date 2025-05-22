use crate::editor::fileinfo::FileInfo;

use super::{Location, line::Line};
use std::fs::{self, File};
use std::io::{Error, Write};

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_info: FileInfo,
    pub dirty: bool,
}

/// Where the text resides
impl Buffer {
    /// Checks if a buffer is empty
    pub fn is_empty(&self) -> bool {
        self.height() == 0
    }

    /// Loads a file into a buffer
    pub fn load(filename: &str) -> Result<Self, std::io::Error> {
        let file_contents = fs::read_to_string(filename)?;
        let mut lines = Vec::new();

        for line in file_contents.lines() {
            lines.push(Line::from(line));
        }

        Ok(Self {
            lines,
            file_info: FileInfo::from(filename),
            dirty: false,
        })
    }

    /// Returns the length of buffer lines
    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn insert_char(&mut self, char: char, at: Location) {
        if at.line_index > self.height() {
            return;
        }

        if at.line_index == self.height() {
            self.lines.push(Line::from(&char.to_string()));
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert_char(char, at.grapheme_index);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_index) {
            if at.grapheme_index >= line.grapheme_count()
                && self.height() > at.line_index.saturating_add(1)
            {
                let next_line = self.lines.remove(at.line_index.saturating_add(1));

                // clippy::indexing_slicing: We checked for existence of this line in the surrounding if statement
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_index].append(&next_line);
                self.dirty = true;
            } else if at.grapheme_index < line.grapheme_count() {
                // clippy::indexing_slicing: We checked for existence of this line in the surrounding if statement
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_index].delete(at.grapheme_index);
                self.dirty = true;
            }
        }
    }

    pub fn insert_newline(&mut self, at: Location) {
        if at.line_index == self.height() {
            self.lines.push(Line::default());
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            let new = line.split(at.grapheme_index);
            self.lines.insert(at.line_index.saturating_add(1), new);
            self.dirty = true;
        }
    }

    pub fn save_to_disk(&mut self) -> Result<(), Error> {
        if let Some(path) = &self.file_info.path {
            let mut file = File::create(path)?;

            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
        }

        self.dirty = false;

        Ok(())
    }
}

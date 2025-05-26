use log::info;

use super::{Line, Location};
use crate::editor::fileinfo::FileInfo;
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
        if at.line_idx > self.height() {
            return;
        }

        if at.line_idx == self.height() {
            self.lines.push(Line::from(&char.to_string()));
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_idx) {
            line.insert_char(char, at.grapheme_idx);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_idx) {
            if at.grapheme_idx >= line.grapheme_count()
                && self.height() > at.line_idx.saturating_add(1)
            {
                let next_line = self.lines.remove(at.line_idx.saturating_add(1));

                // clippy::indexing_slicing: We checked for existence of this line in the surrounding if statement
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_idx].append(&next_line);
                self.dirty = true;
            } else if at.grapheme_idx < line.grapheme_count() {
                // clippy::indexing_slicing: We checked for existence of this line in the surrounding if statement
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_idx].delete(at.grapheme_idx);
                self.dirty = true;
            }
        }
    }

    pub fn insert_newline(&mut self, at: Location) {
        if at.line_idx == self.height() {
            self.lines.push(Line::default());
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_idx) {
            let new = line.split(at.grapheme_idx);
            self.lines.insert(at.line_idx.saturating_add(1), new);
            self.dirty = true;
        }
    }

    pub fn save_to_file(&self, file_info: &FileInfo) -> Result<(), Error> {
        if let Some(path) = &file_info.get_path() {
            let mut file = File::create(path)?;

            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
        }

        Ok(())
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        let file_info = FileInfo::from(file_name);
        self.save_to_file(&file_info)?;
        self.file_info = file_info;
        self.dirty = false;

        Ok(())
    }

    pub fn save(&mut self) -> Result<(), Error> {
        self.save_to_file(&self.file_info)?;
        self.dirty = false;

        Ok(())
    }

    pub fn search(&self, query: &str, from: Location) -> Option<Location> {
        info!("buffer search {from:?}");
        for (line_idx, line) in self.lines.iter().enumerate().skip(from.line_idx){
            let from_grapheme_idx = if line_idx == from.line_idx {
                from.grapheme_idx
            } else {
                0
            };

            info!("{line}");

            if let Some(grapheme_idx) = line.search(query, from_grapheme_idx) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }

        for (line_idx, line) in self
            .lines
            .iter()
            .enumerate()
            .take(from.line_idx.saturating_add(1))
        {
            if let Some(grapheme_idx) = line.search(query, 0) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }

        None
    }

    pub const fn is_file_loaded(&self) -> bool {
        self.file_info.has_path()
    }
}

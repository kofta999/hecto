use super::highlighter::Highlighter;
use super::{Line, Location};
use crate::editor::annotatedstring::AnnotatedString;
use crate::editor::fileinfo::FileInfo;
use crate::prelude::*;
use std::fs::{self, File};
use std::io::{Error, Write};
use std::ops::Range;

#[derive(Default)]
pub struct Buffer {
    lines: Vec<Line>,
    file_info: FileInfo,
    dirty: bool,
}

/// Where the text resides
impl Buffer {
    /// Checks if a buffer is empty
    pub fn is_empty(&self) -> bool {
        self.height() == 0
    }

    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub const fn get_file_info(&self) -> &FileInfo {
        &self.file_info
    }

    pub fn grapheme_count(&self, idx: LineIdx) -> GraphemeIdx {
        self.lines.get(idx).map_or(0, Line::grapheme_count)
    }

    pub fn width_until(&self, idx: LineIdx, until: GraphemeIdx) -> GraphemeIdx {
        self.lines
            .get(idx)
            .map_or(0, |line| line.width_until(until))
    }

    pub fn get_highlighted_substring(
        &self,
        line_idx: LineIdx,
        range: Range<GraphemeIdx>,
        highlighter: &Highlighter,
    ) -> Option<AnnotatedString> {
        self.lines.get(line_idx).map(|line| {
            line.get_annotated_visible_substr(range, Some(&highlighter.get_annotations(line_idx)))
        })
    }

    pub fn highlight(&self, idx: LineIdx, highlighter: &mut Highlighter) {
        if let Some(line) = self.lines.get(idx) {
            highlighter.highlight(idx, line);
        }
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
    pub fn height(&self) -> LineIdx {
        self.lines.len()
    }

    pub fn insert_char(&mut self, char: char, at: Location) {
        debug_assert!(at.line_idx <= self.height());

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
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Attempting to save with no file path present");
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

    pub fn search_forward(&self, query: &str, from: Location) -> Option<Location> {
        if query.is_empty() {
            return None;
        }

        let mut is_first = true;

        for (line_idx, line) in self
            .lines
            .iter()
            .enumerate()
            .cycle()
            .skip(from.line_idx)
            .take(self.lines.len().saturating_add(1))
        //taking one more, to search the current line twice (once from the middle, once from the start)
        {
            let from_grapheme_idx = if is_first {
                is_first = false;
                from.grapheme_idx
            } else {
                0
            };

            if let Some(grapheme_idx) = line.search_forward(query, from_grapheme_idx) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }

        None
    }

    pub fn search_backward(&self, query: &str, from: Location) -> Option<Location> {
        if query.is_empty() {
            return None;
        }

        let mut is_first = true;

        for (line_idx, line) in self
            .lines
            .iter()
            .enumerate()
            .rev()
            .cycle()
            .skip(
                self.lines
                    .len()
                    .saturating_sub(from.line_idx)
                    .saturating_sub(1),
            )
            .take(self.lines.len().saturating_add(1))
        {
            let from_grapheme_idx = if is_first {
                is_first = false;
                from.grapheme_idx
            } else {
                line.grapheme_count()
            };

            if let Some(grapheme_idx) = line.search_backward(query, from_grapheme_idx) {
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

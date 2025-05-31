mod graphemewidth;
mod textfragment;
use crate::editor::line::graphemewidth::GraphemeWidth;
use std::{
    fmt::{self},
    ops::{Deref, Range},
};
use textfragment::TextFragment;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::annotatedstring::{AnnotatedString, AnnotationType};

pub type GraphemeIdx = usize;
pub type ByteIdx = usize;
pub type ColIdx = usize;

#[derive(Debug, Default)]
/// A line of text fragments
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        Self {
            fragments: Self::str_to_fragments(line_str),
            string: line_str.to_string(),
        }
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        line_str
            .grapheme_indices(true)
            .map(|(idx, g)| TextFragment {
                grapheme: g.to_string(),
                rendered_width: match g.width() {
                    0 | 1 => GraphemeWidth::Half,
                    _ => GraphemeWidth::Full,
                },
                replacement: Line::get_replacement_character(g),
                start_byte_idx: idx,
            })
            .collect()
    }

    fn rebuild_fragments(&mut self) {
        self.fragments = Self::str_to_fragments(&self.string);
    }

    /// Replaces a grapheme with another character for display if needed
    fn get_replacement_character(g: &str) -> Option<char> {
        match g {
            // TODO: Fix
            "\t" => Some(' '),
            " " => None,
            _ if g.width() > 0 && g.trim().is_empty() => Some('␣'),
            _ if g.width() == 0 => {
                let mut chars = g.chars();
                if let Some(c) = chars.next() {
                    if c.is_control() && chars.next().is_none() {
                        return Some('▯');
                    }
                }
                Some('·')
            }
            _ => None,
        }
    }

    /// Gets the graphemes that can be displayed on screen
    pub fn get_visible_graphemes(&self, range: Range<ColIdx>) -> String {
        self.get_annotated_visible_substr(range, None, None)
            .to_string()
    }

    pub fn get_annotated_visible_substr(
        &self,
        range: Range<ColIdx>,
        query: Option<&str>,
        selected_match: Option<GraphemeIdx>,
    ) -> AnnotatedString {
        if range.start >= range.end {
            return AnnotatedString::default();
        }

        let mut result = AnnotatedString::from(&self.string);

        if let Some(query) = query {
            if !query.is_empty() {
                self.find_all(query, 0..self.string.len()).iter().for_each(
                    |(start_byte_idx, grapheme_idx)| {
                        if let Some(selected_match) = selected_match {
                            if *grapheme_idx == selected_match {
                                result.add_annotation(
                                    AnnotationType::SelectedMatch,
                                    *start_byte_idx,
                                    start_byte_idx.saturating_add(query.len()),
                                );

                                return;
                            }
                        }

                        result.add_annotation(
                            AnnotationType::Match,
                            *start_byte_idx,
                            start_byte_idx.saturating_add(query.len()),
                        );
                    },
                );
            }

            let mut fragment_start = self.width();

            for fragment in self.fragments.iter().rev() {
                let fragment_end = fragment_start;
                fragment_start = fragment_start.saturating_sub(fragment.rendered_width.into());

                if fragment_start > range.end {
                    continue;
                }

                if fragment_start < range.end && fragment_end > range.end {
                    result.replace(fragment.start_byte_idx, self.string.len(), "⋯");
                    continue;
                } else if fragment_start == range.end {
                    result.replace(fragment.start_byte_idx, self.string.len(), "");
                    continue;
                }

                if fragment_end <= range.start {
                    result.replace(
                        0,
                        fragment
                            .start_byte_idx
                            .saturating_add(fragment.grapheme.len()),
                        "",
                    );
                    break;
                } else if fragment_start < range.start && fragment_end > range.start {
                    result.replace(
                        0,
                        fragment
                            .start_byte_idx
                            .saturating_add(fragment.grapheme.len()),
                        "⋯",
                    );

                    break;
                }

                // Fragment is fully within range: Apply replacement characters if appropriate
                if fragment_start >= range.start && fragment_end <= range.end {
                    if let Some(replacement) = fragment.replacement {
                        let start_byte_idx = fragment.start_byte_idx;
                        let end_byte_idx = start_byte_idx.saturating_add(fragment.grapheme.len());
                        result.replace(start_byte_idx, end_byte_idx, &replacement.to_string());
                    }
                }
            }
        }

        result
    }

    fn find_all(&self, query: &str, range: Range<ByteIdx>) -> Vec<(ByteIdx, GraphemeIdx)> {
        let end_byte_idx = range.end;
        let start_byte_idx = range.start;

        self.string
            .get(start_byte_idx..end_byte_idx)
            .map_or_else(Vec::new, |substr| {
                substr
                    .match_indices(query)
                    .filter_map(|(relative_start_idx, _)| {
                        let absolute_start_idx = relative_start_idx.saturating_add(start_byte_idx);

                        self.byte_idx_to_grapheme_idx(absolute_start_idx)
                            .map(|grapheme_idx| (absolute_start_idx, grapheme_idx))
                    })
                    .collect()
            })
    }

    /// Returns count of graphemes in the line
    pub fn grapheme_count(&self) -> GraphemeIdx {
        self.fragments.len()
    }

    /// Calculates the width of the current line based on the width of each text fragment
    pub fn width_until(&self, grapheme_idx: GraphemeIdx) -> GraphemeIdx {
        self.fragments
            .iter()
            .take(grapheme_idx)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    /// Inserts a character at a position in the line, or appends it if `at` == `grapheme_count + 1`
    pub fn insert_char(&mut self, char: char, at: GraphemeIdx) {
        debug_assert!(at.saturating_sub(1) <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert(fragment.start_byte_idx, char);
        } else {
            self.string.push(char);
        }

        self.rebuild_fragments();
    }

    /// Deletes a character at a position
    pub fn delete(&mut self, at: GraphemeIdx) {
        debug_assert!(at <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            let start = fragment.start_byte_idx;
            let end = start.saturating_add(fragment.grapheme.len());

            self.string.drain(start..end);
            self.rebuild_fragments();
        }
    }

    /// Appends a Line to self
    pub fn append(&mut self, other: &Self) {
        self.string.push_str(&other.string);
        self.rebuild_fragments();
    }

    /// Splits Line into two
    pub fn split(&mut self, at: GraphemeIdx) -> Self {
        if let Some(fragment) = self.fragments.get(at) {
            let remainder = self.string.split_off(fragment.start_byte_idx);
            self.rebuild_fragments();
            Self::from(&remainder)
        } else {
            Self::default()
        }
    }

    pub fn append_char(&mut self, char: char) {
        self.insert_char(char, self.grapheme_count());
    }

    pub fn delete_last(&mut self) {
        self.delete(self.grapheme_count().saturating_sub(1));
    }

    pub fn width(&self) -> usize {
        self.width_until(self.grapheme_count())
    }

    pub fn search_forward(&self, query: &str, from_grapheme_idx: GraphemeIdx) -> Option<usize> {
        debug_assert!(from_grapheme_idx <= self.grapheme_count());
        if from_grapheme_idx == self.grapheme_count() {
            return None;
        }

        let start_byte_idx = self.grapheme_idx_to_byte_idx(from_grapheme_idx);

        self.find_all(query, start_byte_idx..self.string.len())
            .first()
            .map(|(_, grapheme_idx)| *grapheme_idx)
    }

    pub fn search_backward(&self, query: &str, from_grapheme_idx: GraphemeIdx) -> Option<usize> {
        debug_assert!(from_grapheme_idx <= self.grapheme_count());
        if from_grapheme_idx == 0 {
            return None;
        }

        let end_byte_idx = if from_grapheme_idx == self.grapheme_count() {
            self.string.len()
        } else {
            self.grapheme_idx_to_byte_idx(from_grapheme_idx)
        };

        self.find_all(query, 0..end_byte_idx)
            .last()
            .map(|(_, grapheme_idx)| *grapheme_idx)
    }

    fn byte_idx_to_grapheme_idx(&self, byte_idx: ByteIdx) -> Option<GraphemeIdx> {
        if byte_idx > self.string.len() {
            return None;
        }

        self.fragments
            .iter()
            .position(|fragment| fragment.start_byte_idx >= byte_idx)
    }

    fn grapheme_idx_to_byte_idx(&self, grapheme_idx: GraphemeIdx) -> ByteIdx {
        self.fragments.get(grapheme_idx).map_or_else(
            || {
                #[cfg(debug_assertions)]
                {
                    panic!("Fragment not found for grapheme index: {grapheme_idx:?}");
                }
                #[cfg(not(debug_assertions))]
                {
                    0
                }
            },
            |fragment| fragment.start_byte_idx,
        )
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl Deref for Line {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

mod graphemewidth;
mod textfragment;
use super::annotatedstring::{AnnotatedString, AnnotationType};
use crate::editor::line::graphemewidth::GraphemeWidth;
use crate::prelude::*;
use std::{
    cmp::min,
    fmt::{self},
    ops::{Deref, Range},
};
use textfragment::TextFragment;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
                start: idx,
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
                    |(start, grapheme_idx)| {
                        if let Some(selected_match) = selected_match {
                            if *grapheme_idx == selected_match {
                                result.add_annotation(
                                    AnnotationType::SelectedMatch,
                                    *start,
                                    start.saturating_add(query.len()),
                                );

                                return;
                            }
                        }

                        result.add_annotation(
                            AnnotationType::Match,
                            *start,
                            start.saturating_add(query.len()),
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
                    result.replace(fragment.start, self.string.len(), "⋯");
                    continue;
                } else if fragment_start == range.end {
                    result.truncate_right_from(fragment.start);
                    continue;
                }

                if fragment_end <= range.start {
                    result.truncate_left_until(
                        fragment.start.saturating_add(fragment.grapheme.len()),
                    );
                    break;
                } else if fragment_start < range.start && fragment_end > range.start {
                    result.replace(
                        0,
                        fragment.start.saturating_add(fragment.grapheme.len()),
                        "⋯",
                    );

                    break;
                }

                // Fragment is fully within range: Apply replacement characters if appropriate
                if fragment_start >= range.start && fragment_end <= range.end {
                    if let Some(replacement) = fragment.replacement {
                        let start = fragment.start;
                        let end = start.saturating_add(fragment.grapheme.len());
                        result.replace(start, end, &replacement.to_string());
                    }
                }
            }
        }

        result
    }

    fn find_all(&self, query: &str, range: Range<ByteIdx>) -> Vec<(ByteIdx, GraphemeIdx)> {
        let end = min(range.end, self.string.len());
        let start = range.start;
        debug_assert!(start <= end);
        debug_assert!(start <= self.string.len());

        self.string.get(start..end).map_or_else(Vec::new, |substr| {
            let potential_matches: Vec<ByteIdx> = substr
                .match_indices(query)
                .map(|(relative_start_idx, _)| relative_start_idx.saturating_add(start))
                .collect();

            self.match_grapheme_clusters(&potential_matches, query)
        })
    }

    fn match_grapheme_clusters(
        &self,
        matches: &[ByteIdx],
        query: &str,
    ) -> Vec<(ByteIdx, GraphemeIdx)> {
        let grapheme_count = query.graphemes(true).count();
        matches
            .iter()
            .filter_map(|&start| {
                self.byte_idx_to_grapheme_idx(start)
                    .and_then(|grapheme_idx| {
                        self.fragments
                            .get(grapheme_idx..grapheme_idx.saturating_add(grapheme_count))
                            .and_then(|fragments| {
                                let substring = fragments
                                    .iter()
                                    .map(|fragment| fragment.grapheme.as_str())
                                    .collect::<String>();
                                (query == substring).then_some((start, grapheme_idx))
                            })
                    })
            })
            .collect()
    }

    /// Returns count of graphemes in the line
    pub fn grapheme_count(&self) -> GraphemeIdx {
        self.fragments.len()
    }

    /// Calculates the width of the current line based on the width of each text fragment
    pub fn width_until(&self, grapheme_idx: GraphemeIdx) -> ColIdx {
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
            self.string.insert(fragment.start, char);
        } else {
            self.string.push(char);
        }

        self.rebuild_fragments();
    }

    /// Deletes a character at a position
    pub fn delete(&mut self, at: GraphemeIdx) {
        debug_assert!(at <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            let start = fragment.start;
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
            let remainder = self.string.split_off(fragment.start);
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

    pub fn width(&self) -> ColIdx {
        self.width_until(self.grapheme_count())
    }

    pub fn search_forward(&self, query: &str, from_grapheme_idx: GraphemeIdx) -> Option<usize> {
        debug_assert!(from_grapheme_idx <= self.grapheme_count());
        if from_grapheme_idx == self.grapheme_count() {
            return None;
        }

        let start = self.grapheme_idx_to_byte_idx(from_grapheme_idx);

        self.find_all(query, start..self.string.len())
            .first()
            .map(|(_, grapheme_idx)| *grapheme_idx)
    }

    pub fn search_backward(&self, query: &str, from_grapheme_idx: GraphemeIdx) -> Option<usize> {
        debug_assert!(from_grapheme_idx <= self.grapheme_count());
        if from_grapheme_idx == 0 {
            return None;
        }

        let end = if from_grapheme_idx == self.grapheme_count() {
            self.string.len()
        } else {
            self.grapheme_idx_to_byte_idx(from_grapheme_idx)
        };

        self.find_all(query, 0..end)
            .last()
            .map(|(_, grapheme_idx)| *grapheme_idx)
    }

    fn byte_idx_to_grapheme_idx(&self, byte_idx: ByteIdx) -> Option<GraphemeIdx> {
        if byte_idx > self.string.len() {
            return None;
        }

        self.fragments
            .iter()
            .position(|fragment| fragment.start >= byte_idx)
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
            |fragment| fragment.start,
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

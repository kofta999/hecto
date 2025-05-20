use std::{
    fmt::{self},
    ops::Range,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy)]
enum GraphemeWidth {
    Half,
    Full,
}

impl GraphemeWidth {
    const fn saturating_add(self, other: usize) -> usize {
        match self {
            Self::Full => other.saturating_add(2),
            Self::Half => other.saturating_add(1),
        }
    }
}

#[derive(Debug)]
/// A text unit, which is a grapheme in Unicode world
struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

#[derive(Debug, Default)]
/// A line of text fragments
pub struct Line {
    fragments: Vec<TextFragment>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        Self {
            fragments: Self::str_to_fragments(line_str),
        }
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        line_str
            .graphemes(true)
            .map(|g| TextFragment {
                grapheme: g.to_string(),
                rendered_width: match g.width() {
                    0 | 1 => GraphemeWidth::Half,
                    _ => GraphemeWidth::Full,
                },
                replacement: Line::replace_character(g),
            })
            .collect()
    }

    /// Replaces a grapheme with another character for display if needed
    fn replace_character(g: &str) -> Option<char> {
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
    pub fn get_visible_graphemes(&self, range: Range<usize>) -> String {
        if range.start >= range.end {
            return String::new();
        }

        let mut current_pos: usize = 0;
        let mut res = String::new();

        for fragment in &self.fragments {
            let fragment_end = fragment.rendered_width.saturating_add(current_pos);

            if current_pos >= range.end {
                break;
            }

            if fragment_end > range.start {
                if fragment_end > range.end || current_pos < range.start {
                    res.push('⋯');
                } else if let Some(char) = fragment.replacement {
                    res.push(char);
                } else {
                    res.push_str(&fragment.grapheme);
                }
            }

            current_pos = fragment_end;
        }

        res
    }

    /// Returns count of graphemes in the line
    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
    }

    /// Calculates the width of the current line based on the width of each text fragment
    pub fn width_until(&self, grapheme_index: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    /// Inserts a character at a position
    pub fn insert_char(&mut self, char: char, at: usize) {
        let mut res = String::new();

        for (i, fragment) in self.fragments.iter().enumerate() {
            if at == i {
                res.push(char);
            }

            res.push_str(&fragment.grapheme);
        }

        if at >= self.fragments.len() {
            res.push(char);
        }

        self.fragments = Self::str_to_fragments(&res);
    }

    /// Deletes a character at a position
    pub fn delete(&mut self, at: usize) {
        let mut res = String::new();

        for (i, fragment) in self.fragments.iter().enumerate() {
            if at != i {
                res.push_str(&fragment.grapheme);
            }
        }

        self.fragments = Self::str_to_fragments(&res);
    }

    /// Appends a Line to self
    pub fn append(&mut self, other: &Self) {
        let mut concat = self.to_string();
        concat.push_str(&other.to_string());

        self.fragments = Self::str_to_fragments(&concat);
    }

    /// Splits Line into two
    pub fn split(&mut self, at: usize) -> Self {
        if at > self.fragments.len() {
            return Self::default();
        }

        Self {
            fragments: self.fragments.split_off(at),
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result: String = self
            .fragments
            .iter()
            .map(|fragment| fragment.grapheme.clone())
            .collect();

        write!(f, "{result}")
    }
}

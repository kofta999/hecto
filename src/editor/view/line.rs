use std::ops::Range;
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
struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

#[derive(Debug)]
pub struct Line {
    fragments: Vec<TextFragment>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        Self {
            fragments: line_str
                .graphemes(true)
                .map(|g| TextFragment {
                    grapheme: g.to_string(),
                    rendered_width: match g.width() {
                        0 | 1 => GraphemeWidth::Half,
                        _ => GraphemeWidth::Full,
                    },
                    replacement: Line::replace_character(g),
                })
                .collect(),
        }
    }

    fn replace_character(g: &str) -> Option<char> {
        match g {
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
}

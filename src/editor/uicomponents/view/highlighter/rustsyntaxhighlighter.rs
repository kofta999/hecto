use super::syntaxhighlighter::SyntaxHighlighter;
use crate::{
    editor::{annotation::Annotation, annotationtype::AnnotationType, line::Line},
    prelude::LineIdx,
};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct RustSyntaxHighlighter {
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl RustSyntaxHighlighter {
    fn highlight_digits(line: &Line, result: &mut Vec<Annotation>) {
        for (word_idx, word) in line.split_word_bound_indices() {
            if Self::is_valid_number(word) {
                result.push(Annotation {
                    annotation_type: AnnotationType::Number,
                    start: word_idx,
                    end: word_idx.saturating_add(word.len()),
                });
            }
        }
    }

    fn is_valid_number(word: &str) -> bool {
        if word.is_empty() {
            return false;
        }

        if Self::is_numeric_literal(word) {
            return true;
        }

        let mut chars = word.chars();

        if let Some(first_char) = chars.next() {
            if !first_char.is_ascii_digit() {
                return false;
            }
        }

        let mut seen_dot = false;
        let mut seen_e = false;
        let mut prev_was_digit = true;

        for char in chars {
            match char {
                '0'..='9' => prev_was_digit = true,
                '_' => {
                    if !prev_was_digit {
                        return false;
                    }
                    prev_was_digit = false;
                }
                '.' => {
                    if seen_dot || seen_e || !prev_was_digit {
                        return false;
                    }
                    seen_dot = true;
                    prev_was_digit = false;
                }
                'e' | 'E' => {
                    if seen_e || !prev_was_digit {
                        return false;
                    }
                    seen_e = true;
                    prev_was_digit = false;
                }
                _ => return false,
            }
        }

        prev_was_digit
    }

    fn is_numeric_literal(word: &str) -> bool {
        if word.len() < 3 {
            return false;
        }

        let mut chars = word.chars();

        if chars.next() != Some('0') {
            return false;
        }

        let base = match chars.next() {
            Some('b' | 'B') => 2,
            Some('o' | 'O') => 8,
            Some('x' | 'X') => 16,
            _ => return false,
        };

        chars.all(|char| char.is_digit(base))
    }
}

impl SyntaxHighlighter for RustSyntaxHighlighter {
    fn highlight(&mut self, idx: LineIdx, line: &Line) {
        let mut result = Vec::new();
        Self::highlight_digits(line, &mut result);
        self.highlights.insert(idx, result);
    }

    fn get_annotations(&self, idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&idx)
    }
}

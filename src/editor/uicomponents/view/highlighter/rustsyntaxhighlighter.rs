use unicode_segmentation::UnicodeSegmentation;

use super::syntaxhighlighter::SyntaxHighlighter;
use crate::{
    editor::{annotation::Annotation, annotationtype::AnnotationType, line::Line},
    prelude::LineIdx,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct RustSyntaxHighlighter {
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl RustSyntaxHighlighter {
    fn highlight_digits(line: &Line, result: &mut Vec<Annotation>) {
        for (word_idx, word) in line.split_word_bound_indices() {
            if Self::is_int(word) || Self::is_float(word) || Self::is_scientific_notation(word) {
                result.push(Annotation {
                    annotation_type: AnnotationType::Number,
                    start: word_idx,
                    end: word_idx.saturating_add(word.len()),
                });
            }
        }
    }

    fn is_int(word: &str) -> bool {
        if word.is_empty() {
            return false;
        }
        let mut should_highlight = true;

        for (char_idx, char) in word.chars().enumerate() {
            if char == '_' {
                if char_idx == 0 || char_idx == word.len().saturating_sub(1) {
                    should_highlight = false;
                }
            } else if !char.is_ascii_digit() {
                should_highlight = false;
            }
        }

        should_highlight
    }

    fn is_float(word: &str) -> bool {
        let res: Vec<&str> = word.split('.').collect();

        if res.len() != 2 {
            return false;
        }

        Self::is_int(res[0]) && Self::is_int(res[1])
    }

    fn is_scientific_notation(word: &str) -> bool {
        let res: Vec<&str> = word.split('e').collect();

        if res.len() != 2 {
            return false;
        }

        (Self::is_float(res[0]) || Self::is_int(res[0])) && Self::is_int(res[1])
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

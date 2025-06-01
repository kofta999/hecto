use super::syntaxhighlighter::SyntaxHighlighter;
use crate::{
    editor::{annotation::Annotation, annotationtype::AnnotationType, line::Line},
    prelude::LineIdx,
};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;
const KEYWORDS: [&str; 52] = [
    "break",
    "const",
    "continue",
    "crate",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "async",
    "await",
    "dyn",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    "try",
    "macro_rules",
    "union",
];
const TYPES: [&str; 22] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "Option", "Result", "String", "str", "Vec", "HashMap",
];
const KNOWN_VALUES: [&str; 4] = ["Some", "None", "Ok", "Err"];

#[derive(Default)]
pub struct RustSyntaxHighlighter {
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl RustSyntaxHighlighter {
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

    fn is_keyword(word: &str) -> bool {
        KEYWORDS.contains(&word)
    }

    fn is_type(word: &str) -> bool {
        TYPES.contains(&word)
    }

    fn is_known_literal(word: &str) -> bool {
        KNOWN_VALUES.contains(&word)
    }
}

impl SyntaxHighlighter for RustSyntaxHighlighter {
    fn highlight(&mut self, idx: LineIdx, line: &Line) {
        let mut result = Vec::new();
        for (word_idx, word) in line.split_word_bound_indices() {
            let mut annotation_type = None;

            if Self::is_valid_number(word) {
                annotation_type = Some(AnnotationType::Number);
            } else if Self::is_keyword(word) {
                annotation_type = Some(AnnotationType::Keyword);
            } else if Self::is_type(word) {
                annotation_type = Some(AnnotationType::Type);
            } else if Self::is_known_literal(word) {
                annotation_type = Some(AnnotationType::KnownLiteral);
            }

            if let Some(annotation_type) = annotation_type {
                result.push(Annotation {
                    annotation_type,
                    start: word_idx,
                    end: word_idx.saturating_add(word.len()),
                });
            }
        }

        self.highlights.insert(idx, result);
    }

    fn get_annotations(&self, idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&idx)
    }
}

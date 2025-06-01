use crate::{
    editor::{annotation::Annotation, line::Line},
    prelude::LineIdx,
};

pub trait SyntaxHighlighter {
    fn highlight(&mut self, idx: LineIdx, line: &Line);
    fn get_annotations(&self, idx: LineIdx) -> Option<&Vec<Annotation>>;
}

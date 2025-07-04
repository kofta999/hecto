use super::annotationtype::AnnotationType;
use crate::prelude::ByteIdx;

// clippy::struct_field_names: naming the field `type` is disallowed due to type being a keyword.
#[derive(Copy, Clone, Debug)]
#[allow(clippy::struct_field_names)]
pub struct Annotation {
    pub annotation_type: AnnotationType,
    pub start: ByteIdx,
    pub end: ByteIdx,
}

impl Annotation {
    pub fn shift(&mut self, offset: usize) {
        self.start = self.start.saturating_add(offset);
        self.end = self.end.saturating_add(offset);
    }
}

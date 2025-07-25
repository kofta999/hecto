use crate::editor::annotationtype::AnnotationType;



pub struct AnnotatedStringPart<'a> {
    pub string: &'a str,
    pub annotation_type: Option<AnnotationType>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AnnotationType {
    Match,
    SelectedMatch,
    Number,
    Keyword,
    Type,
    KnownLiteral,
}

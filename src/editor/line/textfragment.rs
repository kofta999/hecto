use super::graphemewidth::GraphemeWidth;

/// A text unit, which is a grapheme in Unicode world
pub struct TextFragment {
    pub grapheme: String,
    pub rendered_width: GraphemeWidth,
    pub replacement: Option<char>,
    pub start: usize,
}

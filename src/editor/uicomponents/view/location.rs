#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub grapheme_idx: usize,
    pub line_idx: usize,
}

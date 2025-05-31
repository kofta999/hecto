use super::{GraphemeIdx, LineIdx};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub grapheme_idx: GraphemeIdx,
    pub line_idx: LineIdx,
}

type Row = usize;
type Col = usize;

#[derive(Default, Clone, Copy)]
pub struct Position {
    pub row: Row,
    pub col: Col,
}

impl Position {
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self {
            col: self.col.saturating_sub(other.col),
            row: self.row.saturating_sub(other.row),
        }
    }
}

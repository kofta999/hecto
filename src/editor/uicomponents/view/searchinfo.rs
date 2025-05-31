use super::location::Location;
use crate::editor::{line::Line, position::Position};

pub struct SearchInfo {
    pub prev_location: Location,
    pub prev_scroll_offset: Position,
    pub query: Option<Line>,
}

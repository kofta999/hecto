use super::super::{
    command::{Edit, Move},
    documentstatus::DocumentStatus,
    line::Line,
    position::Position,
    size::Size,
    terminal::Terminal,
};
use super::UIComponent;
use crate::editor::NAME;
use crate::editor::VERSION;
use buffer::Buffer;
pub use location::Location;
use log::info;
use searchdirection::SearchDirection;
use searchinfo::SearchInfo;
use std::{cmp::min, io::Error};
mod buffer;
mod location;
mod searchdirection;
mod searchinfo;

#[derive(Default)]
pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    text_location: Location,
    scroll_offset: Position,
    search_info: Option<SearchInfo>,
}

impl View {
    // --- Command Handlers ---

    pub fn handle_edit_command(&mut self, command: Edit) {
        match command {
            Edit::Insert(char) => self.insert_char(char),
            Edit::InsertNewLine => self.insert_newline(),
            Edit::DeleteBackward => self.delete_backward(),
            Edit::Delete => self.delete(),
        }
    }

    pub fn handle_move_command(&mut self, command: Move) {
        let Size { height, .. } = self.size;

        match command {
            Move::Up => self.move_up(1),
            Move::Down => self.move_down(1),
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::PageUp => self.move_up(height.saturating_sub(1)),
            Move::PageDown => self.move_down(height.saturating_sub(1)),
            Move::StartOfLine => self.move_to_start_of_line(),
            Move::EndOfLine => self.move_to_end_of_line(),
        }

        self.scroll_text_location_into_view();
    }

    // --- File Operations ---

    pub fn is_file_loaded(&self) -> bool {
        self.buffer.is_file_loaded()
    }

    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        self.buffer = Buffer::load(filename)?;
        self.set_needs_redraw(true);

        Ok(())
    }

    pub fn save(&mut self) -> Result<(), Error> {
        self.buffer.save()
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        self.buffer.save_as(file_name)
    }

    // --- Cursor / Location Management ---

    pub fn caret_position(&self) -> Position {
        self.text_location_into_position()
            .saturating_sub(self.scroll_offset)
    }

    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            filename: format!("{}", self.buffer.file_info),
            line_count: self.buffer.height(),
            text_location: self.text_location,
            is_modified: self.buffer.dirty,
        }
    }

    fn move_up(&mut self, step: usize) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    // clippy::arithmetic_side_effects: This function performs arithmetic calculations
    // after explicitly checking that the target value will be within bounds.
    #[allow(clippy::arithmetic_side_effects)]
    fn move_right(&mut self) {
        let line_width = self.get_line_width(self.text_location.line_idx);

        if self.text_location.grapheme_idx < line_width {
            self.text_location.grapheme_idx += 1;
        } else {
            self.move_to_start_of_line();
            self.move_down(1);
        }
    }

    // clippy::arithmetic_side_effects: This function performs arithmetic calculations
    // after explicitly checking that the target value will be within bounds.
    #[allow(clippy::arithmetic_side_effects)]
    fn move_left(&mut self) {
        if self.text_location.grapheme_idx > 0 {
            self.text_location.grapheme_idx -= 1;
        } else if self.text_location.line_idx > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }

    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_idx = 0;
    }

    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_idx = self.get_line_width(self.text_location.line_idx);
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_idx = self
            .get_line_width(self.text_location.line_idx)
            .min(self.text_location.grapheme_idx);
    }

    fn snap_to_valid_line(&mut self) {
        self.text_location.line_idx = self.text_location.line_idx.min(self.buffer.height());
    }

    fn text_location_into_position(&self) -> Position {
        let row = self.text_location.line_idx;
        debug_assert!(row.saturating_sub(1) <= self.buffer.lines.len());
        let col = self
            .buffer
            .lines
            .get(row)
            .map_or(0, |line| line.width_until(self.text_location.grapheme_idx));

        Position { row, col }
    }

    fn get_line_width(&self, at: usize) -> usize {
        self.buffer.lines.get(at).map_or(0, Line::grapheme_count)
    }

    fn insert_char(&mut self, char: char) {
        let old_len = self.get_line_width(self.text_location.line_idx);
        self.buffer.insert_char(char, self.text_location);
        let new_len = self.get_line_width(self.text_location.line_idx);

        let delta = new_len.saturating_sub(old_len);

        if delta > 0 {
            self.handle_move_command(Move::Right);
        }

        self.set_needs_redraw(true);
    }

    fn delete_backward(&mut self) {
        self.handle_move_command(Move::Left);

        if self.text_location.grapheme_idx == 0 && self.text_location.line_idx == 0 {
            return;
        }

        self.buffer.delete(self.text_location);
        self.set_needs_redraw(true);
    }

    fn delete(&mut self) {
        if self.text_location.grapheme_idx == self.get_line_width(self.text_location.line_idx)
            && self.text_location.line_idx == self.buffer.height()
        {
            return;
        }

        self.buffer.delete(self.text_location);
        self.set_needs_redraw(true);
    }

    fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.text_location);
        self.handle_move_command(Move::Right);
        self.set_needs_redraw(true);
    }

    // --- Scrolling ---

    fn scroll_text_location_into_view(&mut self) {
        let Position { row, col } = self.text_location_into_position();

        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }

    fn scroll_vertically(&mut self, to: usize) {
        let Size { height, .. } = self.size;

        let offset_changed = if to < self.scroll_offset.row {
            self.scroll_offset.row = to;
            true
        } else if to >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = to.saturating_sub(height).saturating_add(1);
            true
        } else {
            false
        };

        if offset_changed {
            self.set_needs_redraw(true);
        }
    }

    fn scroll_horizontally(&mut self, to: usize) {
        let Size { width, .. } = self.size;

        let offset_changed = if to < self.scroll_offset.col {
            self.scroll_offset.col = to;
            true
        } else if to >= self.scroll_offset.row.saturating_add(width) {
            self.scroll_offset.col = to.saturating_sub(width).saturating_add(1);
            true
        } else {
            false
        };

        if offset_changed {
            self.set_needs_redraw(true);
        }
    }

    fn center_text_location(&mut self) {
        let Size { height, width } = self.size;
        let Position { row, col } = self.text_location_into_position();
        let vertical_mid = height.div_ceil(2);
        let horizontal_mid = width.div_ceil(2);
        self.scroll_offset.row = row.saturating_sub(vertical_mid);
        self.scroll_offset.col = col.saturating_sub(horizontal_mid);
        self.set_needs_redraw(true);
    }

    // --- Rendering Helpers ---

    fn render_line(at: usize, line_text: &str) -> Result<(), Error> {
        Terminal::print_row(at, line_text)
    }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let welcome_message = format!("{NAME} editor -- Version {VERSION}");
        let len = welcome_message.len();
        let remaining_width = width.saturating_sub(1);

        if remaining_width <= len {
            return String::from("~");
        }

        format!("{:<1}{:^remaining_width$}", "~", welcome_message)
    }

    // --- Search ---

    pub fn enter_search(&mut self) {
        self.search_info = Some(SearchInfo {
            prev_location: self.text_location,
            prev_scroll_offset: self.scroll_offset,
            query: None,
        });
    }

    pub fn exit_search(&mut self) {
        self.search_info = None;
        self.set_needs_redraw(true);
    }

    pub fn dismiss_search(&mut self) {
        if let Some(search_info) = &self.search_info {
            self.text_location = search_info.prev_location;
            self.scroll_offset = search_info.prev_scroll_offset;
            self.scroll_text_location_into_view();
        }

        self.search_info = None;
        self.set_needs_redraw(true);
    }

    pub fn search(&mut self, query: &str) {
        if let Some(search_info) = self.search_info.as_mut() {
            search_info.query = Some(Line::from(query));
        }
        self.search_in_direction(self.text_location, SearchDirection::default());
        self.set_needs_redraw(true);
    }

    /// Panics on debug if query not found
    fn get_search_query(&self) -> Option<&Line> {
        let query = self
            .search_info
            .as_ref()
            .and_then(|search_info| search_info.query.as_ref());

        debug_assert!(
            query.is_some(),
            "Attempting to search with malformed searchinfo present"
        );

        query
    }

    pub fn search_in_direction(&mut self, from: Location, direction: SearchDirection) {
        if let Some(location) = self.get_search_query().and_then(|query| {
            if query.is_empty() {
                return None;
            }

            if direction == SearchDirection::Backward {
                self.buffer.search_backward(query, from)
            } else {
                self.buffer.search_forward(query, from)
            }
        }) {
            self.text_location = location;
            self.center_text_location();
        }
    }

    pub fn search_next(&mut self) {
        let step_right = self
            .get_search_query()
            .map_or(1, |query| min(query.grapheme_count(), 1));

        let location = Location {
            line_idx: self.text_location.line_idx,
            grapheme_idx: self.text_location.grapheme_idx.saturating_add(step_right),
        };

        self.search_in_direction(location, SearchDirection::Forward);
    }

    pub fn search_prev(&mut self) {
        info!("{:?}", self.text_location);
        self.search_in_direction(self.text_location, SearchDirection::Backward);
    }
}

impl UIComponent for View {
    fn draw(&mut self, origin_row: usize) -> Result<(), Error> {
        let Size { height, width } = self.size;

        let top_third = height.div_ceil(3);
        let scroll_top = self.scroll_offset.row;
        let end_y = origin_row.saturating_add(height);

        for current_row in origin_row..end_y {
            let line_idx = current_row
                .saturating_sub(origin_row)
                .saturating_add(scroll_top);

            if let Some(line) = self.buffer.lines.get(line_idx) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);

                let query = self
                    .search_info
                    .as_ref()
                    .and_then(|search_info| search_info.query.as_deref());

                let selected_match = (self.text_location.line_idx == line_idx && query.is_some())
                    .then_some(self.text_location.grapheme_idx);

                Terminal::print_annotated_row(
                    current_row,
                    &line.get_annotated_visible_substr(left..right, query, selected_match),
                )?;
            } else if current_row == top_third && self.buffer.is_empty() {
                let message = Self::build_welcome_message(width);
                Self::render_line(current_row, &message)?;
            } else {
                Self::render_line(current_row, "~")?;
            }
        }

        Ok(())
    }

    fn set_size(&mut self, to: Size) {
        self.size = to;
        self.scroll_text_location_into_view();
    }

    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&mut self) -> bool {
        self.needs_redraw
    }
}

use super::{
    command::{Edit, Move},
    documentstatus::DocumentStatus,
    line::Line,
    position::Position,
    size::Size,
    terminal::Terminal,
    uicomponent::UIComponent,
};
use crate::editor::NAME;
use crate::editor::VERSION;
use buffer::Buffer;

use std::io::Error;
mod buffer;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

#[derive(Default)]
pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    text_location: Location,
    scroll_offset: Position,
}

impl View {
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
        // This match moves the positon, but does not check for all boundaries.
        // The final boundarline checking happens after the match statement.

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

    pub fn is_file_loaded(&self) -> bool {
        self.buffer.is_file_loaded()
    }

    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        self.buffer = Buffer::load(filename)?;
        self.set_needs_redraw(true);

        Ok(())
    }

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
        self.text_location.line_index = self.text_location.line_index.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    // clippy::arithmetic_side_effects: This function performs arithmetic calculations
    // after explicitly checking that the target value will be within bounds.
    #[allow(clippy::arithmetic_side_effects)]
    fn move_right(&mut self) {
        let line_width = self.get_line_width(self.text_location.line_index);

        if self.text_location.grapheme_index < line_width {
            self.text_location.grapheme_index += 1;
        } else {
            self.move_to_start_of_line();
            self.move_down(1);
        }
    }

    // clippy::arithmetic_side_effects: This function performs arithmetic calculations
    // after explicitly checking that the target value will be within bounds.
    #[allow(clippy::arithmetic_side_effects)]
    fn move_left(&mut self) {
        if self.text_location.grapheme_index > 0 {
            self.text_location.grapheme_index -= 1;
        } else if self.text_location.line_index > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }

    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_index = 0;
    }

    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_index = self.get_line_width(self.text_location.line_index);
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .get_line_width(self.text_location.line_index)
            .min(self.text_location.grapheme_index);
    }

    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index = self.text_location.line_index.min(self.buffer.height());
    }

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

    fn render_line(at: usize, line_text: &str) -> Result<(), Error> {
        Terminal::print_row(at, line_text)
    }

    fn text_location_into_position(&self) -> Position {
        let row = self.text_location.line_index;
        let col = self.buffer.lines.get(row).map_or(0, |line| {
            line.width_until(self.text_location.grapheme_index)
        });

        Position { row, col }
    }

    fn get_line_width(&self, at: usize) -> usize {
        self.buffer.lines.get(at).map_or(0, Line::grapheme_count)
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

    fn insert_char(&mut self, char: char) {
        let old_len = self.get_line_width(self.text_location.line_index);
        self.buffer.insert_char(char, self.text_location);
        let new_len = self.get_line_width(self.text_location.line_index);

        let delta = new_len.saturating_sub(old_len);

        if delta > 0 {
            self.handle_move_command(Move::Right);
        }

        self.set_needs_redraw(true);
    }

    fn delete_backward(&mut self) {
        self.handle_move_command(Move::Left);

        if self.text_location.grapheme_index == 0 && self.text_location.line_index == 0 {
            return;
        }

        self.buffer.delete(self.text_location);
        self.set_needs_redraw(true);
    }

    fn delete(&mut self) {
        if self.text_location.grapheme_index == self.get_line_width(self.text_location.line_index)
            && self.text_location.line_index == self.buffer.height()
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

    pub fn save(&mut self) -> Result<(), Error> {
        self.buffer.save()
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        self.buffer.save_as(file_name)
    }
}

impl UIComponent for View {
    fn draw(&mut self, origin_row: usize) -> Result<(), Error> {
        let Size { height, width } = self.size;

        // Idc if the message wasn't 100% centered
        #[allow(clippy::integer_division)]
        let top_third = height / 3;
        let scroll_top = self.scroll_offset.row;
        let end_y = origin_row.saturating_add(height);

        for current_row in origin_row..end_y {
            let line_idx = current_row
                .saturating_sub(origin_row)
                .saturating_add(scroll_top);

            if let Some(line) = self.buffer.lines.get(line_idx) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);

                Self::render_line(current_row, &line.get_visible_graphemes(left..right))?;
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

use std::io::Error;

use super::{
    NAME, VERSION,
    documentstatus::DocumentStatus,
    editorcommand::{Direction, EditorCommand, InsertionType},
    terminal::{Position, Size, Terminal},
    uicomponent::UIComponent,
};
use buffer::Buffer;
use line::Line;
mod buffer;
mod line;

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
    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_text_location(direction),
            EditorCommand::Insert(InsertionType::Char(char)) => self.insert_char(char),
            EditorCommand::Insert(InsertionType::Newline) => self.insert_newline(),
            EditorCommand::Delete(Direction::Left) => self.delete_left(),
            EditorCommand::Delete(Direction::Right) => self.delete_right(),
            // Only supports left and right deletions for now
            EditorCommand::Delete(_) => (),
            EditorCommand::Save => self.save_file(),
            // Ignored
            EditorCommand::Resize(_) | EditorCommand::Quit => {}
        }
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.set_needs_redraw(true);
        }
    }

    pub fn caret_position(&self) -> Position {
        self.text_location_into_position()
            .saturating_sub(self.scroll_offset)
    }

    pub fn move_text_location(&mut self, direction: Direction) {
        let Size { height, .. } = Terminal::size().unwrap_or_default();

        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),

            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_sub(1)),
            Direction::LeftSide => self.move_to_start_of_line(),
            Direction::RightSide => self.move_to_end_of_line(),
        }

        self.scroll_text_location_into_view();
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
            self.move_text_location(Direction::Right);
        }

        self.set_needs_redraw(true);
    }

    fn delete_left(&mut self) {
        self.move_text_location(Direction::Left);

        if self.text_location.grapheme_index == 0 && self.text_location.line_index == 0 {
            return;
        }

        self.buffer.delete(self.text_location);
        self.set_needs_redraw(true);
    }

    fn delete_right(&mut self) {
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
        self.move_text_location(Direction::Right);
        self.set_needs_redraw(true);
    }

    fn save_file(&mut self) {
        let _ = self.buffer.save_to_disk();
    }
}

impl UIComponent for View {
    fn draw(&mut self, origin_y: usize) -> Result<(), Error> {
        let Size { height, width } = self.size;

        // Idc if the message wasn't 100% centered
        #[allow(clippy::integer_division)]
        let top_third = height / 3;
        let scroll_top = self.scroll_offset.row;
        let end_y = origin_y.saturating_add(height);

        for current_row in origin_y..end_y {
            let line_idx = current_row
                .saturating_sub(origin_y)
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

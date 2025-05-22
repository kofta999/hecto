use super::{
    NAME, VERSION,
    documentstatus::DocumentStatus,
    editorcommand::{Direction, EditorCommand, InsertionType},
    terminal::{Position, Size, Terminal},
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
    margin_bottom: usize,
    text_location: Location,
    scroll_offset: Position,
}

impl View {
    pub fn new(margin_bottom: usize) -> Self {
        let terminal_size = Terminal::size().unwrap_or_default();

        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Size {
                height: terminal_size.height.saturating_sub(margin_bottom),
                width: terminal_size.width,
            },
            margin_bottom,
            text_location: Location::default(),
            scroll_offset: Position::default(),
        }
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_text_location(direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Insert(InsertionType::Char(char)) => self.insert_char(char),
            EditorCommand::Insert(InsertionType::Newline) => self.insert_newline(),
            EditorCommand::Delete(Direction::Left) => self.delete_left(),
            EditorCommand::Delete(Direction::Right) => self.delete_right(),
            // Only supports left and right deletions for now
            EditorCommand::Delete(_) => (),
            EditorCommand::Save => self.save_file(),
            EditorCommand::Quit => {}
        }
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
        }
    }

    pub fn render(&mut self) {
        if !self.needs_redraw || self.size.height == 0 {
            return;
        }

        let Size { height, width } = Terminal::size().unwrap_or_default();
        if height == 0 || width == 0 {
            return;
        }

        // Idc if the message wasn't 100% centered
        #[allow(clippy::integer_division)]
        let vertical_center = height / 3;
        let top = self.scroll_offset.row;

        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);

                Self::render_line(current_row, &line.get_visible_graphemes(left..right));
            } else if current_row == vertical_center && self.buffer.is_empty() {
                let message = Self::build_welcome_message(width);
                Self::render_line(current_row, &message);
            } else {
                Self::render_line(current_row, "~");
            }
        }

        self.needs_redraw = false;
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

    fn resize(&mut self, to: Size) {
        self.size = Size {
            width: to.width,
            height: to.height.saturating_sub(self.margin_bottom),
        };
        self.scroll_text_location_into_view();
        self.needs_redraw = true;
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

        self.needs_redraw = self.needs_redraw || offset_changed;
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

        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok());
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

        self.needs_redraw = true;
    }

    fn delete_left(&mut self) {
        self.move_text_location(Direction::Left);

        if self.text_location.grapheme_index == 0 && self.text_location.line_index == 0 {
            return;
        }

        self.buffer.delete(self.text_location);
        self.needs_redraw = true;
    }

    fn delete_right(&mut self) {
        if self.text_location.grapheme_index == self.get_line_width(self.text_location.line_index)
            && self.text_location.line_index == self.buffer.height()
        {
            return;
        }

        self.buffer.delete(self.text_location);
        self.needs_redraw = true;
    }

    fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.text_location);
        self.move_text_location(Direction::Right);
        self.needs_redraw = true;
    }

    fn save_file(&mut self) {
        let _ = self.buffer.save_to_disk();
    }
}

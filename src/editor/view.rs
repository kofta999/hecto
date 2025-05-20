use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
};
use buffer::Buffer;
use line::Line;
mod buffer;
mod line;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Clone, Copy, Debug)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

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
            EditorCommand::Move(direction) => self.move_text_location(&direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Insert(char) => self.insert_char(char),
            EditorCommand::Quit => {}
        }
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
        }
    }

    pub fn render(&self) {
        if !self.needs_redraw {
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
    }

    pub fn caret_position(&self) -> Position {
        self.text_location_into_position()
            .saturating_sub(self.scroll_offset)
    }

    pub fn move_text_location(&mut self, direction: &Direction) {
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
            self.move_to_start_of_line();
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
        self.size = to;
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

        let mut welcome_message = format!("{NAME} editor -- Version {VERSION}");
        let len = welcome_message.len();

        if width <= len {
            return String::from("~");
        }

        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len).saturating_sub(1)) / 2;
        let full_message = format!("~{}{}", " ".repeat(padding), welcome_message);
        welcome_message.truncate(width);

        full_message
    }

    fn insert_char(&mut self, char: char) {
        let old_len = self.get_line_width(self.text_location.line_index);
        self.buffer.insert_char(char, self.text_location);
        let new_len = self.get_line_width(self.text_location.line_index);

        let delta = new_len.saturating_sub(old_len);

        if delta > 0 {
            self.move_right();
        }

        self.needs_redraw = true;
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            text_location: Location::default(),
            scroll_offset: Position::default(),
        }
    }
}

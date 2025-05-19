use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
};
use buffer::Buffer;
use line::Line;
use location::Location;
use log::info;
mod buffer;
mod line;
mod location;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    location: Location,
    scroll_offset: Location,
}

impl View {
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
        let top = self.scroll_offset.y;

        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                let left = self.scroll_offset.x;
                let right = self.scroll_offset.x.saturating_add(width);

                Self::render_line(current_row, &line.get(left..right));
            } else if current_row == vertical_center && self.buffer.is_empty() {
                let message = Self::build_welcome_message(width);
                Self::render_line(current_row, &message);
            } else {
                Self::render_line(current_row, "~");
            }
        }
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_text_location(&direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Quit => {}
        }
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
        }
    }

    pub fn get_position(&self) -> Position {
        self.location.subtract(self.scroll_offset).into()
    }

    pub fn move_text_location(&mut self, direction: &Direction) {
        let Location { mut x, mut y } = self.location;
        let Size { height, .. } = Terminal::size().unwrap_or_default();
        let current_line_len = self.get_line_len(y);
        let buffer_len = self.buffer.lines.len();
        info!("before movement: {current_line_len:?}");

        match direction {
            Direction::Up => y = y.saturating_sub(1),
            Direction::Down => y = y.saturating_add(1).min(buffer_len),
            Direction::Left => x = x.saturating_sub(1),
            Direction::Right => x = x.saturating_add(1).min(current_line_len.unwrap_or(0)),
            Direction::PageUp => y = y.saturating_sub(height),
            Direction::PageDown => y = y.saturating_add(height).min(buffer_len),
            Direction::LeftSide => x = 0,
            Direction::RightSide => x = current_line_len.unwrap_or(0),
        }

        let next_line_len = self.get_line_len(y);
        info!("after movement: {next_line_len:?}");

        if next_line_len.unwrap_or(0) < current_line_len.unwrap_or(0) {
            x = x.min(next_line_len.unwrap_or(0));
        }

        self.location = Location { x, y };
        self.scroll_location_into_view();
    }

    fn resize(&mut self, to: Size) {
        self.size = to;
        self.needs_redraw = true;
    }

    fn scroll_location_into_view(&mut self) {
        let Location { x, y } = self.location;
        let Size { height, width } = self.size;
        let mut offset_changed = false;

        if y < self.scroll_offset.y {
            self.scroll_offset.y = y;
            offset_changed = true;
        } else if y >= self.scroll_offset.y.saturating_add(height) {
            self.scroll_offset.y = y.saturating_sub(height).saturating_add(1);
            offset_changed = true;
        }

        if x < self.scroll_offset.x {
            self.scroll_offset.x = x;
        } else if x >= self.scroll_offset.x.saturating_add(width) {
            self.scroll_offset.x = x.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }

        self.needs_redraw = offset_changed;
    }

    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok());
    }

    fn get_line_len(&self, at: usize) -> Option<usize> {
        self.buffer.lines.get(at).map(Line::len)
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
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            location: Location::default(),
            scroll_offset: Location::default(),
        }
    }
}

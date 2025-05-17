use super::{
    buffer::Buffer,
    terminal::{Size, Terminal},
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
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

        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row) {
                let truncated_line = if line.len() >= width {
                    &line[0..width]
                } else {
                    line
                };

                Self::render_line(current_row, truncated_line);
            } else if current_row == vertical_center && self.buffer.is_empty() {
                let message = Self::build_welcome_message(width);
                Self::render_line(current_row, &message);
            } else {
                Self::render_line(current_row, "~");
            }
        }
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
        }
    }

    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.needs_redraw = true;
    }

    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok());
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
        }
    }
}

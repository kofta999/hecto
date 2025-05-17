use super::{
    buffer::Buffer,
    terminal::{Position, Size, Terminal},
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct View {
    buffer: Buffer,
    pub needs_redraw: bool,
}

impl View {
    pub fn render(&self) -> Result<(), std::io::Error> {
        if self.buffer.is_empty() {
            Self::render_welcome_message()
        } else {
            self.render_buffer()
        }
    }

    pub fn render_welcome_message() -> Result<(), std::io::Error> {
        let Size { height, .. } = Terminal::size()?;

        for current_row in 0..height {
            Terminal::move_caret_to(&Position {
                row: current_row,
                col: 0,
            })?;
            Terminal::clear_line()?;

            Terminal::print("~")?;
        }

        Self::draw_welcome_message()?;

        Ok(())
    }

    pub fn render_buffer(&self) -> Result<(), std::io::Error> {
        let Size { height, width } = Terminal::size()?;

        for current_row in 0..height {
            Terminal::move_caret_to(&Position {
                row: current_row,
                col: 0,
            })?;
            Terminal::clear_line()?;

            if let Some(str) = self.buffer.lines.get(current_row) {
                let mut s = str.clone();
                s.truncate(width);

                Terminal::print(&s)?;
            } else {
                Terminal::print("~")?;
            }
        }

        Ok(())
    }

    pub fn draw_welcome_message() -> Result<(), std::io::Error> {
        let size = Terminal::size()?;
        let mut welcome_message = format!("{NAME} editor -- Version {VERSION}");

        welcome_message.truncate(size.width);

        #[allow(clippy::as_conversions)]
        #[allow(clippy::arithmetic_side_effects)]
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::integer_division)]
        let position = Position {
            col: (size.width - welcome_message.len()) / 2,
            row: size.height / 3,
        };

        Terminal::move_caret_to(&position)?;
        Terminal::print(&welcome_message)
    }

    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
        }
    }
}

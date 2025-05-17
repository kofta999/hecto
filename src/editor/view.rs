use super::terminal::{Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {}

impl View {
    pub fn render() -> Result<(), std::io::Error> {
        let Size { height, .. } = Terminal::size()?;

        for current_row in 0..height {
            Terminal::clear_line()?;

            if current_row == 0 {
                Terminal::print("Hello World!")?;
            } else {
                Terminal::print("~")?;
            }

            if current_row.saturating_add(1) < height {
                Terminal::print("\r\n")?;
            }
        }

        Self::draw_welcome_message()?;

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
            row: (size.width - welcome_message.len()) / 2,
            col: size.height / 3,
        };

        Terminal::move_caret_to(&position)?;
        Terminal::print(&welcome_message)
    }
}

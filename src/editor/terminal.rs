use std::io::{self, Error, Write};

use crossterm::{Command, cursor, queue, style, terminal};

#[derive(Default)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}

#[derive(Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

pub enum PointMovements {
    Up,
    Down,
    Left,
    Right,
    TopSide,
    BottomSide,
    LeftSide,
    RightSide,
}

pub struct Terminal;

impl Terminal {
    pub fn initialize() -> Result<(), Error> {
        terminal::enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_caret_to(&Position { row: 0, col: 0 })?;
        Self::execute()
    }

    pub fn terminate() -> Result<(), Error> {
        terminal::disable_raw_mode()
    }

    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(terminal::Clear(terminal::ClearType::All))
    }

    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(terminal::Clear(terminal::ClearType::CurrentLine))
    }

    pub fn move_caret_to(position: &Position) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        // It's fine to convert here, u16 < usize on Desktop systems (32/64bits)
        Self::queue_command(cursor::MoveTo(position.col as u16, position.row as u16))
    }

    pub fn size() -> Result<Size, Error> {
        let (width, height) = terminal::size()?;

        #[allow(clippy::as_conversions)]
        // It's fine to convert here, u16 < usize on Desktop systems (32/64bits)
        Ok(Size {
            height: height as usize,
            width: width as usize,
        })
    }

    pub fn hide_caret() -> Result<(), Error> {
        Self::queue_command(cursor::Hide)
    }

    pub fn show_caret() -> Result<(), Error> {
        Self::queue_command(cursor::Show)
    }

    pub fn print(s: &str) -> Result<(), Error> {
        Self::queue_command(style::Print(s))
    }

    pub fn execute() -> Result<(), Error> {
        io::stdout().flush()
    }

    pub fn queue_command(command: impl Command) -> Result<(), Error> {
        queue!(io::stdout(), command)
    }
}

use std::{
    fmt::Display,
    io::{self, Error, Write},
};

use crossterm::{Command, cursor, queue, style, terminal};

pub struct Size {
    pub height: u16,
    pub width: u16,
}

pub struct Position {
    pub x: u16,
    pub y: u16,
}

pub struct Terminal;

impl Terminal {
    pub fn initialize() -> Result<(), Error> {
        terminal::enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(&Position { x: 0, y: 0 })?;
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

    pub fn move_cursor_to(position: &Position) -> Result<(), Error> {
        Self::queue_command(cursor::MoveTo(position.x, position.y))
    }

    pub fn size() -> Result<Size, Error> {
        let (width, height) = terminal::size()?;

        Ok(Size { height, width })
    }

    pub fn hide_cursor() -> Result<(), Error> {
        Self::queue_command(cursor::Hide)
    }

    pub fn show_cursor() -> Result<(), Error> {
        Self::queue_command(cursor::Show)
    }

    pub fn print<T: Display>(s: T) -> Result<(), Error> {
        Self::queue_command(style::Print(s))
    }

    pub fn execute() -> Result<(), Error> {
        io::stdout().flush()
    }

    pub fn queue_command(command: impl Command) -> Result<(), Error> {
        queue!(io::stdout(), command)
    }
}

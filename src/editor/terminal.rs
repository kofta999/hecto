use std::{
    fmt::Display,
    io::{self, Write},
};

use crossterm::{cursor, queue, style, terminal};

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
    pub fn initialize() -> Result<(), std::io::Error> {
        terminal::enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(&Position { x: 0, y: 0 })?;
        Self::execute()
    }

    pub fn terminate() -> Result<(), std::io::Error> {
        terminal::disable_raw_mode()
    }

    pub fn clear_screen() -> Result<(), std::io::Error> {
        queue!(io::stdout(), terminal::Clear(terminal::ClearType::All))
    }

    pub fn clear_line() -> Result<(), io::Error> {
        queue!(
            io::stdout(),
            terminal::Clear(terminal::ClearType::CurrentLine)
        )
    }

    pub fn move_cursor_to(position: &Position) -> Result<(), io::Error> {
        queue!(io::stdout(), cursor::MoveTo(position.x, position.y))
    }

    pub fn size() -> Result<Size, io::Error> {
        let (width, height) = terminal::size()?;

        Ok(Size { height, width })
    }

    pub fn hide_cursor() -> Result<(), io::Error> {
        queue!(io::stdout(), cursor::Hide)
    }

    pub fn show_cursor() -> Result<(), io::Error> {
        queue!(io::stdout(), cursor::Show)
    }

    pub fn print<T: Display>(s: T) -> Result<(), io::Error> {
        queue!(io::stdout(), style::Print(s))
    }

    pub fn execute() -> Result<(), io::Error> {
        io::stdout().flush()
    }
}

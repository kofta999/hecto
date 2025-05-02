use std::io;

use crossterm::{cursor, execute, terminal};

pub struct Terminal {}

impl Terminal {
    pub fn initialize() -> Result<(), std::io::Error> {
        terminal::enable_raw_mode()?;
        Self::clear_screen()
    }

    pub fn terminate() -> Result<(), std::io::Error> {
        terminal::disable_raw_mode()
    }

    pub fn clear_screen() -> Result<(), std::io::Error> {
        execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))
    }

    pub fn move_cursor_to(x: u16, y: u16) -> Result<(), io::Error> {
        execute!(io::stdout(), cursor::MoveTo(x, y))
    }

    pub fn size() -> Result<(u16, u16), io::Error> {
        terminal::size()
    }
}

mod attribute;
use super::annotatedstring::AnnotatedString;
use crate::prelude::*;
use attribute::Attribute;
use crossterm::{
    Command, cursor, queue,
    style::{
        self,
        Attribute::{Reset, Reverse},
        ResetColor,
    },
    terminal,
};
use std::io::{self, Error, Write};

pub struct Terminal;

impl Terminal {
    pub fn initialize() -> Result<(), Error> {
        terminal::enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::disable_line_wrap()?;
        Self::clear_screen()?;
        Self::execute()?;

        Ok(())
    }

    pub fn terminate() -> Result<(), Error> {
        Self::leave_alternate_screen()?;
        Self::enable_line_wrap()?;
        Self::show_caret()?;
        Self::execute()?;
        terminal::disable_raw_mode()?;

        Ok(())
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

    pub fn set_title(to: &str) -> Result<(), Error> {
        Self::queue_command(terminal::SetTitle(to))
    }

    pub fn execute() -> Result<(), Error> {
        io::stdout().flush()
    }

    pub fn print_row(row: RowIdx, line_text: &str) -> Result<(), Error> {
        Self::move_caret_to(&Position { row, col: 0 })?;
        Self::clear_line()?;
        Self::print(line_text)
    }

    pub fn print_inverted_row(row: RowIdx, line_text: &str) -> Result<(), Error> {
        let width = Self::size()?.width;
        Self::print_row(row, &format!("{Reverse}{line_text:width$.width$}{Reset}",))
    }

    pub fn print_annotated_row(
        row: RowIdx,
        annotated_string: &AnnotatedString,
    ) -> Result<(), Error> {
        Self::move_caret_to(&Position { row, col: 0 })?;
        Self::clear_line()?;
        annotated_string
            .into_iter()
            .try_for_each(|part| -> Result<(), Error> {
                if let Some(annotation_type) = part.annotation_type {
                    let attribute: Attribute = annotation_type.into();
                    Self::set_attribute(&attribute)?;
                }

                Self::print(part.string)?;
                Self::reset_color()?;

                Ok(())
            })?;

        Ok(())
    }

    fn set_attribute(attribute: &Attribute) -> Result<(), Error> {
        if let Some(foreground_color) = attribute.foreground {
            Self::queue_command(style::SetForegroundColor(foreground_color))?;
        }

        if let Some(background_color) = attribute.background {
            Self::queue_command(style::SetBackgroundColor(background_color))?;
        }

        Ok(())
    }

    fn reset_color() -> Result<(), Error> {
        Self::queue_command(ResetColor)?;
        Ok(())
    }

    fn enter_alternate_screen() -> Result<(), Error> {
        Self::queue_command(terminal::EnterAlternateScreen)
    }

    fn leave_alternate_screen() -> Result<(), Error> {
        Self::queue_command(terminal::LeaveAlternateScreen)
    }

    fn enable_line_wrap() -> Result<(), Error> {
        Self::queue_command(terminal::EnableLineWrap)
    }

    fn disable_line_wrap() -> Result<(), Error> {
        Self::queue_command(terminal::DisableLineWrap)
    }

    fn queue_command(command: impl Command) -> Result<(), Error> {
        queue!(io::stdout(), command)
    }
}

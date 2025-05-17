mod terminal;
mod view;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use terminal::{PointMovements, Position, Size, Terminal};
use view::View;

#[derive(Default)]
pub struct Location {
    x: usize,
    y: usize,
}

#[derive(Default)]
pub struct Editor {
    location: Location,
    should_quit: bool,
}

impl Editor {
    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }

            let event = read()?;
            self.evaluate_event(&event)?;

            Terminal::execute()?;
        }

        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<(), std::io::Error> {
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            match code {
                KeyCode::Char('x') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                // Movements
                KeyCode::Char('h') => self.move_point(&PointMovements::Left)?,
                KeyCode::Char('j') => self.move_point(&PointMovements::Up)?,
                KeyCode::Char('k') => self.move_point(&PointMovements::Down)?,
                KeyCode::Char('l') => self.move_point(&PointMovements::Right)?,

                KeyCode::PageUp => self.move_point(&PointMovements::TopSide)?,
                KeyCode::PageDown => self.move_point(&PointMovements::BottomSide)?,
                KeyCode::Home => self.move_point(&PointMovements::LeftSide)?,
                KeyCode::End => self.move_point(&PointMovements::RightSide)?,
                _ => (),
            }
        }

        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::hide_caret()?;
        Terminal::move_caret_to(&Position::default())?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("またね〜\r\n")?;
        } else {
            View::render()?;

            Terminal::move_caret_to(&Position {
                row: self.location.x,
                col: self.location.y,
            })?;
        }
        Terminal::show_caret()?;

        Terminal::execute()?;

        Ok(())
    }

    pub fn move_point(&mut self, movement: &PointMovements) -> Result<(), std::io::Error> {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::size()?;

        match movement {
            PointMovements::Up => y = y.saturating_add(1).min(height.saturating_sub(1)),

            PointMovements::Down => y = y.saturating_sub(1).min(height.saturating_sub(1)),
            PointMovements::Left => x = x.saturating_sub(1).min(width.saturating_sub(1)),
            PointMovements::Right => x = x.saturating_add(1).min(width.saturating_sub(1)),
            PointMovements::TopSide => y = 0,
            PointMovements::BottomSide => y = height.saturating_sub(1),
            PointMovements::LeftSide => x = 0,
            PointMovements::RightSide => x = width.saturating_sub(1),
        }

        self.location = Location { x, y };

        Ok(())
    }
}

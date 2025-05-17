mod buffer;
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
    view: View,
}

impl Editor {
    pub fn run(&mut self, filename: Option<&String>) {
        Terminal::initialize().unwrap();
        if let Some(file) = filename {
            self.view.load(file);
        }

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
            self.evaluate_event(event)?;

            Terminal::execute()?;
        }

        Ok(())
    }

    // There's no big performance overhead if I passed by value here
    // and this reduces function complexity
    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) -> Result<(), std::io::Error> {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => {
                match (code, modifiers) {
                    (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                        self.should_quit = true;
                    }
                    // Movements
                    (KeyCode::Char('h'), _) => self.move_point(&PointMovements::Left)?,
                    (KeyCode::Char('j'), _) => self.move_point(&PointMovements::Down)?,
                    (KeyCode::Char('k'), _) => self.move_point(&PointMovements::Up)?,
                    (KeyCode::Char('l'), _) => self.move_point(&PointMovements::Right)?,

                    (KeyCode::PageUp, _) => self.move_point(&PointMovements::TopSide)?,
                    (KeyCode::PageDown, _) => self.move_point(&PointMovements::BottomSide)?,
                    (KeyCode::Home, _) => self.move_point(&PointMovements::LeftSide)?,
                    (KeyCode::End, _) => self.move_point(&PointMovements::RightSide)?,
                    _ => (),
                }
            }
            Event::Resize(width_u16, height_u16) => {
                // Will run into problems for rare edge case systems where usize < u16
                #[allow(clippy::as_conversions)]
                self.view.resize(Size {
                    width: width_u16 as usize,
                    height: height_u16 as usize,
                });
            }
            _ => (),
        }

        Ok(())
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Terminal::hide_caret()?;
        Terminal::move_caret_to(&Position::default())?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("またね〜\r\n")?;
        } else {
            self.view.render()?;

            Terminal::move_caret_to(&Position {
                col: self.location.x,
                row: self.location.y,
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
            PointMovements::Up => y = y.saturating_sub(1).min(height.saturating_sub(1)),
            PointMovements::Down => y = y.saturating_add(1).min(height.saturating_sub(1)),
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

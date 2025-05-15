mod terminal;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use terminal::{PointMovements, Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

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
            Self::draw_rows()?;
            Self::draw_welcome_message()?;

            Terminal::move_caret_to(&Position {
                row: self.location.x,
                col: self.location.y,
            })?;
        }
        Terminal::show_caret()?;

        Terminal::execute()?;

        Ok(())
    }

    fn draw_rows() -> Result<(), std::io::Error> {
        let Size { height, .. } = Terminal::size()?;

        for current_row in 0..height {
            Terminal::clear_line()?;
            Terminal::print('~')?;
            if current_row.saturating_add(1) < height {
                Terminal::print("\r\n")?;
            }
        }

        Ok(())
    }

    fn draw_welcome_message() -> Result<(), std::io::Error> {
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
        Terminal::print(welcome_message)
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

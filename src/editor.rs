mod terminal;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};
use terminal::{Position, Size, Terminal};

#[derive(Default)]
pub struct Editor {
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
            self.evaluate_event(&event);

            Terminal::execute()?;
        }

        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                KeyCode::Char('x') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => (),
            }
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("またね〜\r\n")?;
        } else {
            Self::draw_rows()?;
            Terminal::move_cursor_to(&Position { x: 0, y: 0 })?;
        }
        Terminal::show_cursor()?;

        Terminal::execute()?;

        Ok(())
    }

    fn draw_rows() -> Result<(), std::io::Error> {
        let Size { height, .. } = Terminal::size()?;

        for current_row in 0..height {
            Terminal::clear_line()?;
            Terminal::print('~')?;
            if current_row + 1 < height {
                Terminal::print("\r\n")?;
            }
        }

        Ok(())
    }
}

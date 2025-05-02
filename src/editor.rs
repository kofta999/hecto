mod terminal;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};
use terminal::Terminal;

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
        if self.should_quit {
            Terminal::clear_screen()?;
            print!("またね〜\r\n");
        } else {
            Self::draw_rows()?;
            Terminal::move_cursor_to(1, 1)?;
        }

        Ok(())
    }

    fn draw_rows() -> Result<(), std::io::Error> {
        let (_, height) = Terminal::size()?;

        for current_row in 0..height {
            print!("~");
            if current_row + 1 < height {
                print!("\r\n");
            }
        }

        Ok(())
    }
}

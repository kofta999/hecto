mod buffer;
mod terminal;
mod view;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use terminal::{PointMovements, Position, Size, Terminal};
use view::View;

type Result<T> = std::result::Result<T, std::io::Error>;

#[derive(Default, Clone, Copy)]
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
    pub fn new(filename: Option<&String>) -> Result<Self> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;

        let mut view = View::default();

        if let Some(file) = filename {
            view.load(file);
        }

        // Note, using Default::default() here breaks raw mode
        // no questions asked
        Ok(Self {
            view,
            should_quit: false,
            location: Location::default(),
        })
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }

            match read() {
                Ok(event) => self.evaluate_event(event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}")
                    }
                }
            }
        }
    }

    // There's no big performance overhead if I passed by value here
    // and this reduces function complexity
    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
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
                    (KeyCode::Char('h'), _) => self.move_point(&PointMovements::Left),
                    (KeyCode::Char('j'), _) => self.move_point(&PointMovements::Down),
                    (KeyCode::Char('k'), _) => self.move_point(&PointMovements::Up),
                    (KeyCode::Char('l'), _) => self.move_point(&PointMovements::Right),

                    (KeyCode::PageUp, _) => self.move_point(&PointMovements::TopSide),
                    (KeyCode::PageDown, _) => self.move_point(&PointMovements::BottomSide),
                    (KeyCode::Home, _) => self.move_point(&PointMovements::LeftSide),
                    (KeyCode::End, _) => self.move_point(&PointMovements::RightSide),
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
    }

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        self.view.render();
        let _ = Terminal::move_caret_to(&Position {
            col: self.location.x,
            row: self.location.y,
        });
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }

    pub fn move_point(&mut self, movement: &PointMovements) {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::size().unwrap_or_default();

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
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("またね〜\r\n");
        }
    }
}

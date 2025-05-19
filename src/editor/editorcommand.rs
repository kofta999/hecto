use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::terminal::Size;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    TopSide,
    BottomSide,
    LeftSide,
    RightSide,
}

pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Quit,
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => {
                match (code, modifiers) {
                    (KeyCode::Char('x'), KeyModifiers::CONTROL) => Ok(Self::Quit),

                    // Movements
                    (KeyCode::Char('h'), _) => Ok(Self::Move(Direction::Left)),
                    (KeyCode::Char('j'), _) => Ok(Self::Move(Direction::Down)),
                    (KeyCode::Char('k'), _) => Ok(Self::Move(Direction::Up)),
                    (KeyCode::Char('l'), _) => Ok(Self::Move(Direction::Right)),

                    (KeyCode::PageUp, _) => Ok(Self::Move(Direction::TopSide)),
                    (KeyCode::PageDown, _) => Ok(Self::Move(Direction::BottomSide)),
                    (KeyCode::Home, _) => Ok(Self::Move(Direction::LeftSide)),
                    (KeyCode::End, _) => Ok(Self::Move(Direction::RightSide)),
                    _ => Err(format!("KeyCode not supported: {code:?}")),
                }
            }
            Event::Resize(width_u16, height_u16) => {
                // Will run into problems for rare edge case systems where usize < u16
                #[allow(clippy::as_conversions)]
                Ok(Self::Resize(Size {
                    width: width_u16 as usize,
                    height: height_u16 as usize,
                }))
            }
            _ => Err(format!("Event not supported: {event:?}")),
        }
    }
}

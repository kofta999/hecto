use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::terminal::Size;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    LeftSide,
    RightSide,
}

pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Quit,
    Insert(char),
    Delete(Direction),
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

                    (KeyCode::PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                    (KeyCode::PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                    (KeyCode::Home | KeyCode::Char('0'), _) => Ok(Self::Move(Direction::LeftSide)),
                    (KeyCode::End | KeyCode::Char('$'), _) => Ok(Self::Move(Direction::RightSide)),

                    // Insertion
                    (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        Ok(Self::Insert(c))
                    }

                    // Deletion
                    (KeyCode::Backspace, _) => Ok(Self::Delete(Direction::Left)),
                    (KeyCode::Delete, _) => Ok(Self::Delete(Direction::Right)),

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

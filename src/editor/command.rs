use super::terminal::Size;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    StartOfLine,
    EndOfLine,
}

impl TryFrom<KeyEvent> for Move {
    type Error = String;

    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;

        if modifiers == KeyModifiers::NONE {
            match code {
                KeyCode::Char('h') | KeyCode::Left => Ok(Self::Left),
                KeyCode::Char('j') | KeyCode::Down => Ok(Self::Down),
                KeyCode::Char('k') | KeyCode::Up => Ok(Self::Up),
                KeyCode::Char('l') | KeyCode::Right => Ok(Self::Right),

                KeyCode::PageUp => Ok(Self::PageUp),
                KeyCode::PageDown => Ok(Self::PageDown),
                KeyCode::Home | KeyCode::Char('0') => Ok(Self::StartOfLine),
                KeyCode::End | KeyCode::Char('$') => Ok(Self::EndOfLine),

                _ => Err(format!("Unsupported code: {code:?}")),
            }
        } else {
            Err(format!(
                "Unsupported key code {code:?} or modifier {modifiers:?}"
            ))
        }
    }
}

#[derive(Clone, Copy)]
pub enum Edit {
    Insert(char),
    InsertNewLine,
    Delete,
    DeleteBackward,
}

impl TryFrom<KeyEvent> for Edit {
    type Error = String;
    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        match (event.code, event.modifiers) {
            // Insertion
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => Ok(Self::Insert(c)),
            (KeyCode::Tab, _) => Ok(Self::Insert('\t')),
            (KeyCode::Enter, _) => Ok(Self::InsertNewLine),

            // Deletion
            (KeyCode::Backspace, _) => Ok(Self::Delete),
            (KeyCode::Delete, _) => Ok(Self::DeleteBackward),

            _ => Err(format!(
                "Unsupported key code {:?} with modifiers {:?}",
                event.code, event.modifiers
            )),
        }
    }
}

#[derive(Clone, Copy)]
pub enum System {
    Save,
    Resize(Size),
    Quit,
}

impl TryFrom<KeyEvent> for System {
    type Error = String;

    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;

        if modifiers == KeyModifiers::CONTROL {
            match code {
                KeyCode::Char('x') => Ok(Self::Quit),
                KeyCode::Char('s') => Ok(Self::Save),
                _ => Err(format!("Unsupported CONTROL+{code:?} combination")),
            }
        } else {
            Err(format!(
                "Unsupported key code {code:?} or modifier {modifiers:?}"
            ))
        }
    }
}

pub enum Command {
    Move(Move),
    Edit(Edit),
    System(System),
}

// clippy::as_conversions: Will run into problems for rare edge case systems where usize < u16
#[allow(clippy::as_conversions)]
impl TryFrom<Event> for Command {
    type Error = String;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(key_event) => Edit::try_from(key_event)
                .map(Self::Edit)
                .or_else(|_| {
                    Move::try_from(key_event)
                        .map(Self::Move)
                        .or_else(|_| System::try_from(key_event).map(Self::System))
                })
                .map_err(|_err| format!("Event not supported: {key_event:?}")),

            Event::Resize(width_u16, height_u16) => Ok(Self::System(System::Resize(Size {
                height: height_u16 as usize,
                width: width_u16 as usize,
            }))),
            _ => Err(format!("Event not supported: {event:?}")),
        }
    }
}

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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

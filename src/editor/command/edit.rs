use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
            (KeyCode::Backspace, _) => Ok(Self::DeleteBackward),
            (KeyCode::Delete, _) => Ok(Self::Delete),

            _ => Err(format!(
                "Unsupported key code {:?} with modifiers {:?}",
                event.code, event.modifiers
            )),
        }
    }
}

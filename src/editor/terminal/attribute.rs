use crate::editor::annotationtype::AnnotationType;
use crossterm::style::Color;

pub struct Attribute {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}

impl From<AnnotationType> for Attribute {
    fn from(value: AnnotationType) -> Self {
        match value {
            AnnotationType::Match => Self {
                foreground: Some(Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                }),
                background: Some(Color::Rgb {
                    r: 211,
                    g: 211,
                    b: 211,
                }),
            },
            AnnotationType::SelectedMatch => Self {
                foreground: Some(Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                }),
                background: Some(Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 153,
                }),
            },
            AnnotationType::Number => Self {
                foreground: Some(Color::Rgb {
                    r: 255,
                    g: 99,
                    b: 71,
                }),
                background: None,
            },
            AnnotationType::Keyword => Self {
                foreground: Some(Color::Blue),
                background: None,
            },
            AnnotationType::Type => Self {
                foreground: Some(Color::Yellow),
                background: None,
            },
            AnnotationType::KnownLiteral => Self {
                foreground: Some(Color::Magenta),
                background: None,
            },
            AnnotationType::Char => Self {
                foreground: Some(Color::Green),
                background: None,
            },
            AnnotationType::LifetimeSpecifier => Self {
                foreground: Some(Color::DarkMagenta),
                background: None,
            },
            AnnotationType::Comment => Self {
                foreground: Some(Color::DarkGrey),
                background: None,
            },
        }
    }
}

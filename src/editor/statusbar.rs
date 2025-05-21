use super::{terminal::Size, view::Location};
use crate::editor::terminal::Terminal;

pub const STATUSBAR_HEIGHT: u8 = 2;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct FileInfo {
    pub filename: Option<String>,
    pub line_count: usize,
    pub text_location: Location,
    pub is_modified: bool,
}

#[derive(Default)]
pub struct StatusBar {
    file_info: FileInfo,
    margin_bottom: usize,
    needs_redraw: bool,
    width: usize,
    position_y: usize,
}

impl StatusBar {
    pub fn new(margin_bottom: usize) -> Self {
        let terminal_size = Terminal::size().unwrap_or_default();

        Self {
            needs_redraw: true,
            file_info: FileInfo::default(),
            width: terminal_size.width,
            position_y: terminal_size
                .height
                .saturating_sub(margin_bottom)
                .saturating_sub(1),
            margin_bottom,
        }
    }

    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }

        let FileInfo {
            filename,
            line_count,
            text_location,
            is_modified,
        } = &self.file_info;

        let status = format!(
            "{} - Total {} {}:{} {}",
            filename.clone().unwrap_or(String::from("NO NAME")),
            line_count,
            text_location.line_index,
            text_location.grapheme_index,
            if *is_modified { "Modified" } else { "" }
        );

        let result = Terminal::print_row(self.position_y, &status);
        debug_assert!(result.is_ok());
        self.needs_redraw = false;
    }

    pub fn resize(&mut self, size: Size) {
        self.width = size.width;
        self.position_y = size
            .height
            .saturating_sub(self.margin_bottom)
            .saturating_sub(1);

        self.needs_redraw = true;
    }

    pub fn update_info(&mut self, file_info: FileInfo) {
        if file_info != self.file_info {
            self.file_info = file_info;
            self.needs_redraw = true;
        }
    }
}

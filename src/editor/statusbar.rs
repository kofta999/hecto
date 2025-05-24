use super::{documentstatus::DocumentStatus, size::Size, uicomponent::UIComponent};
use crate::editor::terminal::Terminal;
use std::io::Error;

#[derive(Default)]
pub struct StatusBar {
    current_status: DocumentStatus,
    needs_redraw: bool,
    size: Size,
}

impl StatusBar {
    pub fn update_status(&mut self, file_info: DocumentStatus) {
        if file_info != self.current_status {
            self.current_status = file_info;
            self.needs_redraw = true;
        }
    }
}

impl UIComponent for StatusBar {
    fn draw(&mut self, origin_row: usize) -> Result<(), Error> {
        let line_count = self.current_status.line_count_to_string();
        let modified_indicator = self.current_status.modified_indicator_to_string();

        let beginning = format!(
            "{} - {line_count} {modified_indicator}",
            self.current_status.filename
        );

        let position_indicator = self.current_status.position_indicator_to_string();
        let reminder_len = self.size.width.saturating_sub(beginning.len());

        let status = format!("{beginning}{position_indicator:>reminder_len$}");

        let to_print = if status.len() <= self.size.width {
            status
        } else {
            String::new()
        };

        Terminal::print_inverted_row(origin_row, &to_print)
    }

    fn set_size(&mut self, to: Size) {
        self.size = to;
    }

    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&mut self) -> bool {
        self.needs_redraw
    }
}

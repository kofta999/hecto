use super::{documentstatus::DocumentStatus, terminal::Size};
use crate::editor::terminal::Terminal;

#[derive(Default)]
pub struct StatusBar {
    current_status: DocumentStatus,
    margin_bottom: usize,
    needs_redraw: bool,
    width: usize,
    position_y: usize,
    is_visible: bool,
}

impl StatusBar {
    pub fn new(margin_bottom: usize) -> Self {
        let terminal_size = Terminal::size().unwrap_or_default();

        let mut status_bar = Self {
            needs_redraw: true,
            current_status: DocumentStatus::default(),
            width: terminal_size.width,
            position_y: terminal_size
                .height
                .saturating_sub(margin_bottom)
                .saturating_sub(1),
            margin_bottom,
            is_visible: false,
        };

        status_bar.resize(terminal_size);

        status_bar
    }

    pub fn render(&mut self) {
        if !self.needs_redraw || !self.is_visible {
            return;
        }

        if let Ok(size) = Terminal::size() {
            let line_count = self.current_status.line_count_to_string();
            let modified_indicator = self.current_status.modified_indicator_to_string();

            let beginning = format!(
                "{} - {line_count} {modified_indicator}",
                self.current_status.filename
            );

            let position_indicator = self.current_status.position_indicator_to_string();
            let reminder_len = size.width.saturating_sub(beginning.len());

            let status = format!("{beginning}{position_indicator:>reminder_len$}");

            let to_print = if status.len() <= size.width {
                status
            } else {
                String::new()
            };

            let result = Terminal::print_inverted_row(self.position_y, &to_print);
            debug_assert!(result.is_ok(), "Failed to render status bar");
            self.needs_redraw = false;
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.width = size.width;
        let mut position_y = 0;
        let mut is_visible = false;

        if let Some(result) = size
            .height
            .checked_sub(self.margin_bottom)
            .and_then(|result| result.checked_sub(1))
        {
            position_y = result;
            is_visible = true;
        }

        self.position_y = position_y;
        self.is_visible = is_visible;
        self.needs_redraw = true;
    }

    pub fn update_status(&mut self, file_info: DocumentStatus) {
        if file_info != self.current_status {
            self.current_status = file_info;
            self.needs_redraw = true;
        }
    }
}

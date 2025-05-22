use std::io::Error;

use super::{
    terminal::{Size, Terminal},
    uicomponent::UIComponent,
};

#[derive(Default)]
pub struct MessageBar {
    needs_redraw: bool,
    current_message: String,
}

impl MessageBar {
    pub fn update_message(&mut self, to: String) {
        if to != self.current_message {
            self.current_message = to;
            self.mark_redraw(true);
        }
    }
}

impl UIComponent for MessageBar {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&mut self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, _: Size) {}

    fn draw(&mut self, origin: usize) -> Result<(), Error> {
        Terminal::print_row(origin, &self.current_message)
    }
}

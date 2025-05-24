use super::{size::Size, terminal::Terminal, uicomponent::UIComponent};
use std::{
    io::Error,
    time::{Duration, Instant},
};

const DEFAULT_DURATION: Duration = Duration::from_secs(2);

pub struct Message {
    content: String,
    time: Instant,
}

impl Message {
    pub fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.time) > DEFAULT_DURATION
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            content: String::new(),
            time: Instant::now(),
        }
    }
}

#[derive(Default)]
pub struct MessageBar {
    needs_redraw: bool,
    current_message: Message,
    cleared_after_expiry: bool,
}

impl MessageBar {
    pub fn update_message(&mut self, to: &str) {
        if to != self.current_message.content {
            self.current_message = Message {
                content: to.to_string(),
                time: Instant::now(),
            };
            self.cleared_after_expiry = false;
            self.set_needs_redraw(true);
        }
    }
}

impl UIComponent for MessageBar {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&mut self) -> bool {
        (!self.cleared_after_expiry && self.current_message.is_expired()) || self.needs_redraw
    }

    fn set_size(&mut self, _: Size) {}

    fn draw(&mut self, origin: usize) -> Result<(), Error> {
        let message = if self.current_message.is_expired() {
            self.cleared_after_expiry = true;
            ""
        } else {
            &self.current_message.content
        };

        Terminal::print_row(origin, message)
    }
}

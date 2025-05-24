use super::{
    PromptType, command::Edit, line::Line, size::Size, terminal::Terminal, uicomponent::UIComponent,
};

#[derive(Default)]
pub struct CommandBar {
    needs_redraw: bool,
    prompt: String,
    value: Line,
    size: Size,
    prompt_type: PromptType,
}

impl CommandBar {
    pub fn set_prompt(&mut self, prompt_type: PromptType) {
        self.clear_value();
        self.prompt_type = prompt_type;
        self.prompt = match prompt_type {
            PromptType::Save => "Save as: ",
            PromptType::Search => "Search: ",
            PromptType::None => "",
        }
        .into();

        self.set_needs_redraw(true);
    }

    pub fn get_prompt_type(&self) -> PromptType {
        self.prompt_type
    }

    pub fn in_prompt(&self) -> bool {
        !self.prompt_type.is_none()
    }

    pub fn handle_edit_command(&mut self, command: Edit) {
        match command {
            Edit::Insert(c) => self.value.append_char(c),
            Edit::DeleteBackward => self.value.delete_last(),
            Edit::InsertNewLine | Edit::Delete => {}
        }

        self.set_needs_redraw(true);
    }

    pub fn caret_position_col(&self) -> usize {
        self.prompt
            .len()
            .saturating_add(self.value.grapheme_count())
            .min(self.size.width)
    }

    pub fn value(&self) -> String {
        self.value.to_string()
    }

    fn clear_value(&mut self) {
        self.value = Line::default();
        self.set_needs_redraw(true);
    }
}

impl UIComponent for CommandBar {
    fn draw(&mut self, origin_y: usize) -> Result<(), std::io::Error> {
        let area_for_value = self.size.width.saturating_sub(self.prompt.len());
        let value_end = self.value.width();
        let value_start = value_end.saturating_sub(area_for_value);
        let message = format!(
            "{}{}",
            self.prompt,
            self.value.get_visible_graphemes(value_start..value_end)
        );

        let to_print = if message.len() <= self.size.width {
            message
        } else {
            String::new()
        };

        Terminal::print_row(origin_y, &to_print)
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&mut self) -> bool {
        self.needs_redraw
    }
}

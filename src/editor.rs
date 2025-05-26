mod command;
mod commandbar;
mod documentstatus;
mod fileinfo;
mod line;
mod messagebar;
mod position;
mod size;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;
use command::{Command, Edit, Move, System};
use commandbar::CommandBar;
use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use messagebar::MessageBar;
use position::Position;
use size::Size;
use statusbar::StatusBar;
use terminal::Terminal;
use uicomponent::UIComponent;
use view::View;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 3;
const HELP_MESSAGE: &str = "HELP: Ctrl-F = Search | Ctrl-S = save | Ctrl-X = quit";

type Result<T> = std::result::Result<T, std::io::Error>;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum PromptType {
    Search,
    Save,
    #[default]
    None,
}

impl PromptType {
    pub fn is_none(self) -> bool {
        self == Self::None
    }
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: CommandBar,
    title: String,
    terminal_size: Size,
    quit_times: u8,
    prompt_type: PromptType,
}

impl Editor {
    pub fn new(filename: Option<&String>) -> Result<Self> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;

        let size = Terminal::size().unwrap_or_default();
        let mut editor = Self::default();
        editor.handle_resize_command(size);
        editor.message_bar.update_message(HELP_MESSAGE);

        if let Some(file) = filename {
            if editor.view.load(file).is_err() {
                editor
                    .message_bar
                    .update_message(&format!("ERR: Could not open file: {file}"));
            }
        }

        editor.refresh_status();

        Ok(editor)
    }

    pub fn handle_resize_command(&mut self, to: Size) {
        self.terminal_size = to;
        self.view.resize(Size {
            height: to.height.saturating_sub(2),
            width: to.width,
        });

        let bar_size = Size {
            height: 1,
            width: to.width,
        };

        self.status_bar.resize(bar_size);
        self.message_bar.resize(bar_size);
        self.command_bar.resize(bar_size);
    }

    pub fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {NAME}", status.filename);
        self.status_bar.update_status(status);

        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_status();
            self.refresh_screen();
            if self.should_quit {
                break;
            }

            match read() {
                Ok(event) => self.evaluate_event(event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}")
                    }
                }
            }
        }
    }

    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            if let Ok(command) = Command::try_from(event) {
                self.process_command(command);
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn process_command(&mut self, command: Command) {
        if let Command::System(System::Resize(size)) = command {
            self.handle_resize_command(size);
            return;
        }

        match self.prompt_type {
            PromptType::Search => self.process_command_during_search(command),
            PromptType::Save => self.process_command_during_save(command),
            PromptType::None => self.process_command_no_prompt(command),
        }
    }

    fn process_command_no_prompt(&mut self, command: Command) {
        if matches!(command, Command::System(System::Quit)) {
            self.handle_quit_command();
            return;
        }
        self.reset_quit_times();

        match command {
            Command::System(System::Save) => self.handle_save_command(),
            Command::System(System::Search) => self.set_prompt(PromptType::Search),
            Command::Edit(edit_command) => self.view.handle_edit_command(edit_command),
            Command::Move(move_command) => self.view.handle_move_command(move_command),
            Command::System(System::Quit | System::Resize(_) | System::Dismiss) => {}
        }
    }

    fn process_command_during_save(&mut self, command: Command) {
        match command {
            Command::System(System::Quit | System::Resize(_) | System::Search | System::Save)
            | Command::Move(_) => {}
            Command::System(System::Dismiss) => {
                self.set_prompt(PromptType::None);
                self.message_bar.update_message("Save aborted.");
            }
            Command::Edit(Edit::InsertNewLine) => {
                let file_name = self.command_bar.value();
                self.save(Some(&file_name));
                self.set_prompt(PromptType::None);
            }
            Command::Edit(edit_command) => self.command_bar.handle_edit_command(edit_command),
        }
    }

    fn process_command_during_search(&mut self, command: Command) {
        match command {
            Command::System(System::Dismiss) => {
                self.set_prompt(PromptType::None);
                self.view.dismiss_search();
            }
            Command::Edit(Edit::InsertNewLine) => {
                self.set_prompt(PromptType::None);
                self.view.exit_search();
            }
            Command::Edit(edit_command) => {
                self.command_bar.handle_edit_command(edit_command);
                self.view.search(&self.command_bar.value());
            }
            Command::Move(Move::Down | Move::Right) => {
                self.view.search_next();
            }
            Command::System(System::Quit | System::Resize(_) | System::Search | System::Save)
            | Command::Move(_) => {}
        }
    }

    fn set_prompt(&mut self, prompt_type: PromptType) {
        match prompt_type {
            PromptType::None => self.message_bar.set_needs_redraw(true), //Ensures the message bar is properly painted during the next redraw cycle
            PromptType::Save => self.command_bar.set_prompt("Save as: "),
            PromptType::Search => {
                self.view.enter_search();
                self.command_bar
                    .set_prompt("Search (Esc to cancel, Arrows to navigate): ");
            }
        }
        self.command_bar.clear_value();
        self.prompt_type = prompt_type;
    }

    pub fn in_prompt(&self) -> bool {
        !self.prompt_type.is_none()
    }

    fn handle_save_command(&mut self) {
        if self.view.is_file_loaded() {
            self.save(None);
        } else {
            self.set_prompt(PromptType::Save);
        }
    }

    fn save(&mut self, file_name: Option<&str>) {
        let result = if let Some(name) = file_name {
            self.view.save_as(name)
        } else {
            self.view.save()
        };

        match result {
            Ok(()) => self.message_bar.update_message("File saved successfully."),
            Err(_) => self.message_bar.update_message("Error writing file!"),
        }
    }

    // clippy::arithmetic_side_effects: quit_times is guaranteed to be between 0 and QUIT_TIMES
    #[allow(clippy::arithmetic_side_effects)]
    fn handle_quit_command(&mut self) {
        if !self.view.get_status().is_modified || self.quit_times + 1 == QUIT_TIMES {
            self.should_quit = true;
        } else if self.view.get_status().is_modified {
            self.message_bar.update_message(&format!(
                "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                QUIT_TIMES - self.quit_times - 1
            ));
            self.quit_times += 1;
        }
    }

    fn reset_quit_times(&mut self) {
        if self.quit_times > 0 {
            self.quit_times = 0;
            self.message_bar.update_message("");
        }
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }
        let _ = Terminal::hide_caret();
        let bottom_bar_row = self.terminal_size.height.saturating_sub(1);

        if self.in_prompt() {
            self.command_bar.render(bottom_bar_row);
        } else {
            self.message_bar.render(bottom_bar_row);
        }

        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        }

        if self.terminal_size.height > 2 {
            self.view.render(0);
        }

        let new_caret_pos = if self.in_prompt() {
            Position {
                row: bottom_bar_row,
                col: self.command_bar.caret_position_col(),
            }
        } else {
            self.view.caret_position()
        };

        let _ = Terminal::move_caret_to(&new_caret_pos);
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("またね〜\r\n");
        }
    }
}

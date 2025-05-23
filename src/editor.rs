mod command;
mod documentstatus;
mod fileinfo;
mod messagebar;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;
use command::{Command, System};
use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use messagebar::MessageBar;
use statusbar::StatusBar;
use terminal::{Size, Terminal};
use uicomponent::UIComponent;
use view::View;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 3;

type Result<T> = std::result::Result<T, std::io::Error>;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    title: String,
    terminal_size: Size,
    quit_times: u8,
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
        editor.resize(size);
        editor
            .message_bar
            .update_message("HELP: Ctrl-S = save | Ctrl-X = quit");

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

    pub fn resize(&mut self, to: Size) {
        self.terminal_size = to;
        self.view.resize(Size {
            height: to.height.saturating_sub(2),
            width: to.width,
        });

        self.status_bar.resize(Size {
            height: 1,
            width: to.width,
        });
        self.message_bar.resize(Size {
            height: 1,
            width: to.width,
        });
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
            let file_info = self.view.get_status();
            self.status_bar.update_status(file_info);
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
        match command {
            Command::System(System::Quit) => self.handle_quit(),
            Command::System(System::Resize(size)) => self.resize(size),
            _ => self.reset_quit_times(),
        }

        match command {
            Command::System(System::Quit | System::Resize(_)) => {}
            Command::System(System::Save) => self.handle_save(),
            Command::Edit(edit_command) => self.view.handle_edit_command(edit_command),
            Command::Move(move_command) => self.view.handle_move_command(move_command),
        }
    }

    fn handle_save(&mut self) {
        match self.view.save_file() {
            Ok(()) => self.message_bar.update_message("File saved successfully."),
            Err(_) => self.message_bar.update_message("Error writing file!"),
        }
    }

    // clippy::arithmetic_side_effects: quit_times is guaranteed to be between 0 and QUIT_TIMES
    #[allow(clippy::arithmetic_side_effects)]
    fn handle_quit(&mut self) {
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

        self.message_bar
            .render(self.terminal_size.height.saturating_sub(1));

        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        }

        if self.terminal_size.height > 2 {
            self.view.render(0);
        }

        let _ = Terminal::move_caret_to(&self.view.caret_position());
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

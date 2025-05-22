mod documentstatus;
mod editorcommand;
mod fileinfo;
mod messagebar;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;
use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use editorcommand::EditorCommand;
use messagebar::MessageBar;
use statusbar::StatusBar;
use terminal::{Size, Terminal};
use uicomponent::UIComponent;
use view::View;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

type Result<T> = std::result::Result<T, std::io::Error>;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    title: String,
    terminal_size: Size,
}

impl Editor {
    pub fn new(filename: Option<&String>) -> Result<Self> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();

        if let Some(file) = filename {
            editor.view.load(file);
        }

        editor
            .message_bar
            .update_message("HELP: Ctrl-S = save | Ctrl-X = quit".to_string());

        editor.resize(size);
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

    // There's no big performance overhead if I passed by value here
    // and this reduces function complexity
    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            if let Ok(command) = EditorCommand::try_from(event) {
                if matches!(command, EditorCommand::Quit) {
                    self.should_quit = true;
                } else if let EditorCommand::Resize(size) = command {
                    self.resize(size);
                } else {
                    self.view.handle_command(command);
                }
            }
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Received and discarded unsupported or non-press event.");
            }
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

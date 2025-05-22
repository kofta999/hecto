mod editorcommand;
mod statusbar;
mod terminal;
mod documentstatus;
mod fileinfo;
mod view;
use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use editorcommand::EditorCommand;
use statusbar::StatusBar;
use terminal::Terminal;
use view::View;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const STATUSBAR_HEIGHT: u8 = 2;

type Result<T> = std::result::Result<T, std::io::Error>;

pub struct Editor {
    should_quit: bool,
    view: View,
    statusbar: StatusBar,
    title: String,
}

impl Editor {
    pub fn new(filename: Option<&String>) -> Result<Self> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;
        let mut view = View::new(STATUSBAR_HEIGHT.into());
        if let Some(file) = filename {
            view.load(file);
        }

        let mut editor = Self {
            view,
            should_quit: false,
            statusbar: StatusBar::new(1),
            title: String::new(),
        };

        editor.refresh_status();

        // Note, using Default::default() here breaks raw mode
        // no questions asked
        Ok(editor)
    }

    pub fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {NAME}", status.filename);
        self.statusbar.update_status(status);

        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
    }

    pub fn run(&mut self) {
        loop {
            let file_info = self.view.get_status();
            self.statusbar.update_status(file_info);
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
                } else {
                    self.view.handle_command(command);
                    if let EditorCommand::Resize(size) = command {
                        self.statusbar.resize(size);
                    }
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
        let _ = Terminal::hide_caret();
        self.view.render();
        self.statusbar.render();
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

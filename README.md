# Hecto 🦀 - A Minimalist Text Editor in Rust

Hecto is a terminal-based text editor built in Rust, inspired by [Philipp Flenker's Hecto tutorial series](https://philippflenker.com/hecto/). It aims to be a lightweight, efficient, and extensible editor for the command line.

## ✨ Features

- **Core Text Editing:**
  - Insert and delete characters and lines.
  - UTF-8 support with grapheme cluster awareness (thanks to `unicode-segmentation`).
  - Correct rendering of wide characters (thanks to `unicode-width`).
- **File Operations:**
  - Open files from the command line.
  - Save files (Save, Save As).
  - Dirty file indicator (`(modified)`).
  - Prompts for unsaved changes before quitting.
- **Navigation:**
  - Arrow key movement (Up, Down, Left, Right).
  - Page Up / Page Down.
  - Home / End of line.
- **Search:**
  - Incremental search (Ctrl-F).
  - Navigate search results (Arrow keys while searching).
  - Dismiss search (Esc).
- **User Interface:**
  - **Status Bar:** Displays filename, line count, modified status, cursor position, and file type.
  - **Message Bar:** Shows help messages, errors, and confirmations. Clears automatically.
  - **Command Bar:** Used for prompts like "Save as:" and "Search:".
  - **View Pane:** The main text editing area with scrolling.
  - Responsive to terminal resize events.
  - Sets terminal title.
- **Syntax Highlighting:**
  - Basic syntax highlighting for Rust files.
  - Highlights search matches and the currently selected search match.
  - Extensible design for adding more language highlighters.
- **Robustness:**
  - Panic hook to restore terminal state on crashes.
  - Logging to `test.log` for debugging.

## 🚀 Getting Started

### Prerequisites

- Rust toolchain (latest stable recommended). Install via [rustup](https://rustup.rs/).

### Building

1.  Clone the repository:
    ```bash
    git clone https://github.com/<your-username>/hecto.git # Replace with your repo URL
    cd hecto
    ```
2.  Build the project:
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/hecto`.

### Running

- To open an existing file or create a new one:
  ```bash
  ./target/release/hecto my_file.txt
  ```
- To open an empty buffer:
  ```bash
  ./target/release/hecto
  ```
- During development, you can also use `cargo run`:
  ```bash
  cargo run -- my_file.txt
  ```

## ⌨️ Keybindings

- **`Ctrl-S`**: Save the current file. If the file is new, prompts for a filename.
- **`Ctrl-X`**: Quit the editor. If there are unsaved changes, it will prompt you to press `Ctrl-X` multiple times (currently 3) to confirm.
- **`Ctrl-F`**: Enter search mode.
  - Type your query in the command bar.
  - Use **`Arrow Up/Left`** to find the previous match.
  - Use **`Arrow Down/Right`** to find the next match.
  - Press **`Enter`** to exit search mode, keeping the cursor at the current match.
  - Press **`Esc`** to cancel search and return to the previous cursor position.
- **Arrow Keys (`↑`, `↓`, `←`, `→`)**: Move the cursor.
- **`PageUp` / `PageDown`**: Scroll up/down by a page.
- **`Home`**: Move cursor to the start of the current line.
- **`End`**: Move cursor to the end of the current line.
- **`Enter`**:
  - In normal mode: Insert a new line.
  - In "Save as" prompt: Confirm filename and save.
  - In "Search" prompt: Exit search and jump to the current highlighted match.
- **`Backspace`**: Delete character before the cursor.
- **`Delete`**: Delete character at the cursor.
- **`Esc`**:
  - Dismiss "Save as" prompt.
  - Dismiss "Search" prompt and restore previous cursor position/view.
- **Character Keys**: Insert characters.

## 🛠️ Project Structure

The project is organized into several modules within the `src` directory:

- **`main.rs`**: Entry point, argument parsing, and editor initialization.
- **`editor.rs`**: The main `Editor` struct, event loop, command processing, and UI component management.
- **`prelude/`**: Common type aliases (`ByteIdx`, `LineIdx`, etc.) and simple shared structs (`Position`, `Size`, `Location`).
- **`editor/`**: Contains submodules for different editor functionalities:
  - **`terminal/`**: Abstraction over `crossterm` for terminal manipulation (clearing, cursor, colors, etc.).
  - **`command/`**: Defines `Command` enums (`Edit`, `Move`, `System`) and their parsing from `crossterm` events.
  - **`line.rs`**: Represents a single line of text, handling graphemes, width, and operations like insert/delete/split.
  - **`annotatedstring/`**: A string that can hold annotations (e.g., for syntax highlighting), with an iterator for its parts.
  - **`annotation.rs` & `annotationtype.rs`**: Structs for defining text annotations and their types.
  - **`documentstatus.rs`**: Struct to hold and format status information about the document.
  - **`fileinfo.rs` & `filetype.rs`**: Structs for file metadata and determining file types.
  - **`uicomponents/`**: Defines UI elements:
    - `uicomponent.rs`: A trait for common UI component behavior (draw, resize).
    - `view/`: The main text area.
      - `buffer.rs`: Manages the text content (lines of `Line`).
      - `highlighter/`: Logic for syntax highlighting.
        - `syntaxhighlighter.rs`: Trait for syntax highlighters.
        - `rustsyntaxhighlighter.rs`: Rust specific highlighter.
        - `searchresulthighlighter.rs`: Highlights search terms.
    - `statusbar.rs`: Renders the status bar.
    - `messagebar.rs`: Renders temporary messages.
    - `commandbar.rs`: Renders interactive prompts.

## 📚 Dependencies

- **`crossterm`**: For cross-platform terminal manipulation (raw mode, events, styling).
- **`log`** & **`simple-logging`**: For logging application events.
- **`unicode-segmentation`**: To correctly handle Unicode grapheme clusters.
- **`unicode-width`**: To determine the display width of Unicode characters.

## 💡 How It Works (Briefly)

1.  **Initialization**: The `Editor` initializes the terminal, loads a file (if specified), and sets up UI components.
2.  **Event Loop**: The `run()` method enters a loop:
    - Refreshes the screen (draws UI components if needed).
    - Waits for and reads an event from `crossterm` (key press, resize).
    - Converts the event into an internal `Command`.
    - Processes the `Command`:
      - If in a prompt (Save/Search), routes command to prompt-specific logic.
      - Otherwise, routes to general command handling (move, edit, system actions like save/quit).
    - Updates editor state (cursor position, buffer content, UI component status).
3.  **Rendering**: UI components (`View`, `StatusBar`, `MessageBar`, `CommandBar`) are responsible for drawing themselves to the terminal. They only redraw if their content or size has changed. The `View` uses a `Buffer` to store text and a `Highlighter` to get `AnnotatedString`s for display.
4.  **Termination**: When `should_quit` is true, the loop exits. The `Drop` implementation for `Editor` ensures the terminal is restored to its original state.

## 🤝 Contributing

Contributions are welcome! If you'd like to contribute, please feel free to:

1.  Fork the repository.
2.  Create a new branch (`git checkout -b feature/your-feature-name`).
3.  Make your changes.
4.  Commit your changes (`git commit -am 'Add some feature'`).
5.  Push to the branch (`git push origin feature/your-feature-name`).
6.  Open a Pull Request.

Please ensure your code adheres to the existing style and consider adding tests if applicable.

## 📝 TODO

- [ ] Support for more syntax highlighting languages.
- [ ] Configuration file (e.g., TOML) for settings.
- [ ] More advanced editing features (e.g., copy/paste, undo/redo).
- [ ] Mouse support.
- [ ] Basic Vim-like modal editing (Normal, Insert modes).

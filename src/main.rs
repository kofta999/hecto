#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]
use editor::Editor;
mod editor;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    Editor::default().run(args.get(1));
}

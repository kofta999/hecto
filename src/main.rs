#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]
use editor::Editor;
use log::LevelFilter;
mod editor;
mod prelude;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    simple_logging::log_to_file("test.log", LevelFilter::Info).unwrap();

    Editor::new(args.get(1)).unwrap().run();
}

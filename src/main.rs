use app::App;
use crossterm::style::Stylize;
use std::process;

mod app;
mod cli;
mod global;
mod http;
mod module;
mod source;
mod state;
mod utils;

fn main() {
    if let Err(err) = App::run() {
        eprintln!("{} {err:?}", "error:".red());
        process::exit(1);
    }
}

use std::process;

use app::App;
use crossterm::style::Stylize;

mod app;
mod cli;
mod config;
mod module;
mod paths;
mod source;
mod state;
mod users;
mod utils;

fn main() {
    if let Err(err) = App::run() {
        eprintln!("{} {err:?}", "error:".red());
        process::exit(1);
    }
}

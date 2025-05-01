use app::App;
use crossterm::style::Stylize;
use std::process;

mod app;
mod cli;
mod fs;
mod global;
mod globs;
#[cfg(feature = "http")]
mod http;
mod module;
mod source;
mod state;
mod upon;
mod user;
mod utils;

fn main() {
    if let Err(err) = App::run() {
        eprintln!("{} {err:?}", "error:".red());
        process::exit(1);
    }
}

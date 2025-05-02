use app::App;
use crossterm::style::Stylize;
use std::process;

mod app;
mod cli;
mod config;
mod env;
mod fs;
mod globs;
#[cfg(feature = "http")]
mod http;
mod module;
mod source;
mod state;
mod upon;
mod utils;

fn main() {
    if let Err(err) = App::run() {
        eprintln!("{} {err:?}", "error:".red());
        process::exit(1);
    }
}

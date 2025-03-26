use std::process;

use app::App;
use clap::Parser;
use cli::Cli;
use crossterm::style::Stylize;

mod app;
mod cli;
mod config;
mod module;
mod paths;
mod source;
mod state;
mod utils;

fn main() {
    if let Err(error) = App::run(Cli::parse()) {
        eprintln!("{} {error:?}", "error:".red());
        process::exit(1);
    }
}

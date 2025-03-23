use std::process;

use app::App;
use clap::Parser;
use cli::Cli;
use log::error;

mod app;
mod cli;
mod config;
mod link;
mod logging;
mod module;
mod paths;
mod source;
mod state;
mod utils;

fn main() {
    logging::enable();
    if let Err(error) = App::run(Cli::parse()) {
        error!("{error:?}");
        process::exit(1);
    }
}

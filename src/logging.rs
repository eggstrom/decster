use std::io::{self, Write};

use crossterm::style::{StyledContent, Stylize};
use env_logger::{Builder, fmt::Formatter};
use log::{Level, LevelFilter, Record};

pub fn enable() {
    let mut logger = Builder::new();
    logger.format(format);
    #[cfg(debug_assertions)]
    logger.filter_level(LevelFilter::max());
    logger.init();
}

fn format(f: &mut Formatter, record: &Record) -> io::Result<()> {
    writeln!(f, "{} {}", level(record.level()), record.args())
}

fn level(level: Level) -> StyledContent<&'static str> {
    match level {
        Level::Error => "error:".red(),
        Level::Warn => "warn:".yellow(),
        Level::Info => "info:".green(),
        Level::Debug => "debug:".blue(),
        Level::Trace => "trace:".cyan(),
    }
    .bold()
}

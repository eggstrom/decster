use clap::{Arg, arg};

pub fn link_method() -> [Arg; 4] {
    [
        arg!(-s --skip "Skip files that can't be created"),
        arg!(-t --take "Take ownership of existing files if contents match"),
        arg!(-a --ask "Ask whether existing files should be overwritten"),
        arg!(-o --overwrite "Overwrite existing files").conflicts_with_all(["skip", "take", "ask"]),
    ]
}

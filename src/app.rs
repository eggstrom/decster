use std::collections::BTreeSet;

use anyhow::Result;
use clap::Parser;
use crossterm::style::Stylize;

use crate::{
    cli::{Cli, Command, InfoArgs},
    global::{self, config},
    out,
    state::State,
    utils::output::PathExt,
};

pub struct App {
    state: State,
}

impl App {
    pub fn run() -> Result<()> {
        let cli = Cli::parse();
        global::init(&cli)?;
        let app = App {
            state: State::load()?,
        };

        match cli.command {
            Command::Info(args) => app.info(args),
            Command::Enable { modules } => app.enable(modules.into_iter().collect())?,
            Command::Disable { modules } => app.disable(modules.into_iter().collect())?,
            Command::Update { modules } => app.update(modules.into_iter().collect())?,
        }
        Ok(())
    }

    fn info(self, args: InfoArgs) {
        let (modules, filter) = args.modules();
        for (name, module, paths) in self.state.modules(modules, filter) {
            out!("Module {}", name.magenta());
            let files = module.files();
            let hard_links = module.hard_links();
            let symlinks = module.symlinks();

            if files.len() > 0 {
                out!("  Files");
                files.for_each(|link| out!("    {link}"));
            }
            if hard_links.len() > 0 {
                out!("  Hard link");
                hard_links.for_each(|link| out!("    {link}"));
            }
            if symlinks.len() > 0 {
                out!("  Symlinks");
                symlinks.for_each(|link| out!("    {link}"));
            }

            if let Some(paths) = paths {
                out!("  Owned paths");
                for (path, info) in paths {
                    out!("    {}", path.display_kind(info.kind()))
                }
            }
        }
    }

    fn enable(mut self, modules: BTreeSet<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.enable_all_modules();
        } else {
            for module in modules {
                self.state.enable_module(&module);
            }
        }
        self.state.save()
    }

    fn disable(mut self, modules: BTreeSet<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.disable_all_modules();
        } else {
            for module in modules {
                self.state.disable_module(&module);
            }
        }
        self.state.save()
    }

    fn update(mut self, modules: BTreeSet<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.update_all_modules();
        } else {
            for module in modules {
                self.state.update_module(&module);
            }
        }
        self.state.save()
    }
}

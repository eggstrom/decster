use std::collections::BTreeSet;

use anyhow::Result;
use clap::Parser;
use crossterm::style::Stylize;

use crate::{
    cli::{Cli, Command, InfoArgs},
    config, out, paths,
    state::State,
    users::Users,
    utils::output::PathExt,
};

pub struct App {
    users: Users,
    state: State,
}

impl App {
    pub fn run() -> Result<()> {
        let cli = Cli::parse();
        paths::load()?;
        config::load(&cli)?;
        let app = App {
            users: Users::new(),
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
            out!(0, n, "Module {}", name.magenta());
            let files = module.files();
            let hard_links = module.hard_links();
            let symlinks = module.symlinks();

            if files.len() > 0 {
                out!(1, n, "Files");
                files.for_each(|link| out!(2, n, "{link}"));
            }
            if hard_links.len() > 0 {
                out!(1, n, "Hard links");
                hard_links.for_each(|link| out!(2, n, "{link}"));
            }
            if symlinks.len() > 0 {
                out!(1, n, "Symlinks");
                symlinks.for_each(|link| out!(2, n, "{link}"));
            }

            if let Some(paths) = paths {
                out!(1, n, "Owned paths");
                for (path, info) in paths {
                    out!(2, n, "{}", path.display_kind(info.kind()))
                }
            }
        }
    }

    fn enable(mut self, modules: BTreeSet<String>) -> Result<()> {
        if modules.is_empty() {
            self.state.enable_all_modules(&mut self.users);
        } else {
            for module in modules {
                self.state.enable_module(&mut self.users, &module);
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
            self.state.update_all_modules(&mut self.users);
        } else {
            for module in modules {
                self.state.update_module(&mut self.users, &module);
            }
        }
        self.state.save()
    }
}

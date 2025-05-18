use std::borrow::Cow;

use anyhow::Result;
use clap::{ArgMatches, Command, arg};
use termtree::Tree;

use crate::{app::App, globs::Globs};

#[derive(Debug)]
pub struct ShowCli<'a> {
    queries: Vec<&'a str>,
}

impl<'a> ShowCli<'a> {
    pub fn command() -> Command {
        Command::new("show")
            .about("Show current state")
            .arg(arg!([QUERIES]...))
    }

    pub fn parse(matches: &'a ArgMatches) -> Self {
        let queries = matches
            .get_many::<String>("QUERIES")
            .unwrap_or_default()
            .map(|s| s.as_str())
            .collect();
        ShowCli { queries }
    }

    pub fn run(&self, app: App) -> Result<()> {
        let globs = Globs::permissive(&self.queries)?;
        let modules: Vec<_> = app
            .state
            .modules()
            .filter_map(|(name, state)| state.tree(name, &globs))
            .collect();
        let leaves =
            [(!modules.is_empty()).then(|| Tree::new("Modules".into()).with_leaves(modules))]
                .into_iter()
                .flatten();
        let leaves = Vec::from_iter(leaves);

        if !leaves.is_empty() {
            let tree = Tree::<Cow<_>>::new("State".into()).with_leaves(leaves);
            print!("{tree}");
        }
        Ok(())
    }
}

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
        let modules = app
            .state
            .modules()
            .map(|(name, state)| state.tree(name, &globs));
        let tree = Tree::<Cow<_>>::new("State".into())
            .with_leaves([Tree::new("Modules".into()).with_leaves(modules)]);
        print!("{tree}");
        Ok(())
    }
}

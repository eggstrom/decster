use std::borrow::Cow;

use anyhow::Result;
use clap::{ArgMatches, Command, arg};
use crossterm::style::Stylize;
use termtree::Tree;

use crate::{app::App, config, globs::Globs, utils::sha256::Sha256Hash};

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
        let static_sources: Vec<Cow<_>> = config::static_sources()
            .filter(|source| globs.is_match(source))
            .map(|source| {
                let hash = Sha256Hash::from_path(&app.env.static_source_dir().join(source));
                let hash = match hash {
                    Ok(hash) => hash.to_string().yellow().to_string(),
                    Err(err) => format!("{} {err:?}", "error".red()),
                };
                format!("{source}: {hash}").into()
            })
            .collect();

        let leaves = [
            (!modules.is_empty()).then(|| Tree::new("Modules".into()).with_leaves(modules)),
            (!static_sources.is_empty())
                .then(|| Tree::new("Static Sources".into()).with_leaves(static_sources)),
        ]
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

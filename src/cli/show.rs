use std::{borrow::Cow, fmt::Display, path::Path};

use anyhow::Result;
use clap::{ArgMatches, Command, arg};
use crossterm::style::Stylize;
use termtree::Tree;

use crate::{app::App, config, globs::Globs, utils::sha256::Sha256Hash};

pub fn command() -> Command {
    Command::new("show")
        .about("Show current state")
        .arg(arg!([QUERIES]...))
}

pub fn run(app: App, matches: ArgMatches) -> Result<()> {
    let queries: Vec<_> = matches
        .get_many::<String>("QUERIES")
        .unwrap_or_default()
        .map(|s| s.as_str())
        .collect();

    let globs = Globs::permissive(&queries)?;
    let modules: Vec<_> = app
        .state
        .modules()
        .filter_map(|(name, state)| state.tree(name, &globs))
        .collect();

    let leaves = [
        sources(&app, &globs),
        (!modules.is_empty()).then(|| Tree::new("Modules".into()).with_leaves(modules)),
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

fn sources<'a>(app: &'a App, globs: &Globs) -> Option<Tree<Cow<'a, str>>> {
    let static_sources: Vec<Cow<_>> = config::static_sources()
        .filter(|source| globs.is_match(source))
        .map(|name| hash_string(name, &app.env.static_source_dir().join(name)).into())
        .collect();
    let dynamic_sources: Vec<Cow<_>> = app
        .state
        .sources()
        .filter(|ident| ident.matches_globs(globs))
        .map(|ident| hash_string(ident, &ident.path(&app.env)).into())
        .collect();

    let leaves: Vec<_> = [
        (!static_sources.is_empty())
            .then(|| Tree::new("Static".into()).with_leaves(static_sources)),
        (!dynamic_sources.is_empty())
            .then(|| Tree::new("Dynamic".into()).with_leaves(dynamic_sources)),
    ]
    .into_iter()
    .flatten()
    .collect();
    (!leaves.is_empty()).then(|| Tree::new("Sources".into()).with_leaves(leaves))
}

fn hash_string<D>(source: D, path: &Path) -> String
where
    D: Display,
{
    let hash = match Sha256Hash::from_path(path) {
        Ok(hash) => hash.to_string().yellow().to_string(),
        Err(err) => format!("{} {err:?}", "error".red()),
    };
    format!("{source}: {hash}")
}

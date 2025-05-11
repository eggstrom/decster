use std::{fmt::Display, path::Path};

use anyhow::{Result, bail};
use clap::{ArgMatches, Command, arg};
use crossterm::style::Stylize;

use crate::{
    app::App,
    config,
    globs::Globs,
    source::{ident::SourceIdent, name::SourceName},
    utils::{pretty::Pretty, sha256::Sha256Hash},
};

pub struct HashCli<'a> {
    sources: Vec<&'a str>,
}

impl<'a> HashCli<'a> {
    pub fn command() -> Command {
        Command::new("hash")
            .about("Show hashes of fetched sources")
            .arg(arg!([SOURCES]...))
    }

    pub fn parse(matches: &'a ArgMatches) -> Self {
        let sources = matches
            .get_many::<String>("SOURCES")
            .unwrap_or_default()
            .map(|s| s.as_str())
            .collect();
        HashCli { sources }
    }

    pub fn run(&self, app: App) -> Result<()> {
        if self.sources.is_empty() {
            if !self.run_inner(&app, config::static_sources(), app.state.sources()) {
                bail!("There are no fetched sources");
            }
        } else {
            let globs = Globs::permissive(&self.sources)?;
            if !self.run_inner(
                &app,
                config::static_sources_matching_globs(&globs),
                app.state.sources_matching_globs(&globs),
            ) {
                let sources = self.sources.as_slice();
                bail!("{} didn't match any fetched sources", sources.pretty());
            }
        };
        Ok(())
    }

    fn run_inner<'b, 'c, S, D>(&self, app: &App, static_sources: S, dynamic_sources: D) -> bool
    where
        S: Iterator<Item = &'b SourceName>,
        D: Iterator<Item = &'c SourceIdent>,
    {
        let mut has_sources = false;
        for source in static_sources {
            print!("({}) ", "Static".blue());
            Self::print_source_hash(source, &app.env.static_source_dir().join(source));
            has_sources = true;
        }
        for source in dynamic_sources {
            print!("({}) ", "Dynamic".blue());
            Self::print_source_hash(source, &source.path(&app.env));
            has_sources = true;
        }
        has_sources
    }

    fn print_source_hash<D>(ident: D, path: &Path)
    where
        D: Display,
    {
        print!("{ident}: ");
        match Sha256Hash::from_path(path) {
            Ok(hash) => println!("{hash}"),
            Err(err) => println!("{} {err:?}", "error:".red()),
        }
    }
}

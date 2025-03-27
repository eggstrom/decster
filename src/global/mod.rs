use std::sync::OnceLock;

use anyhow::Result;
use config::Config;
use paths::Paths;

use crate::cli::Cli;

pub mod config;
pub mod paths;

struct GlobalState {
    paths: Paths,
    config: Config,
}

impl GlobalState {
    pub fn new(cli: &Cli) -> Result<Self> {
        let paths = Paths::new()?;
        let config = Config::parse(cli, &paths)?;
        Ok(GlobalState { paths, config })
    }
}

static STATE: OnceLock<GlobalState> = OnceLock::new();

pub fn init(cli: &Cli) -> Result<()> {
    STATE
        .set(GlobalState::new(cli)?)
        .ok()
        .expect("`global::init` should only be called once");
    Ok(())
}

fn state() -> &'static GlobalState {
    STATE
        .get()
        .expect("`global::init` should be called without failing before global state is read")
}

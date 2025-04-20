use std::{path::PathBuf, sync::OnceLock};

use anyhow::Result;
use config::Config;
use env::Env;

use crate::cli::Behavior;

pub mod config;
pub mod env;

struct GlobalState {
    env: Env,
    config: Config,
}

impl GlobalState {
    pub fn load(config: Option<PathBuf>, behavior: Behavior) -> Result<Self> {
        let env = Env::load(config)?;
        let config = Config::load(&env, behavior)?;
        Ok(GlobalState { env, config })
    }
}

static STATE: OnceLock<GlobalState> = OnceLock::new();

pub fn load(config: Option<PathBuf>, behavior: Behavior) -> Result<()> {
    STATE
        .set(GlobalState::load(config, behavior)?)
        .ok()
        .expect("`global::load` should only be called once");
    Ok(())
}

fn state() -> &'static GlobalState {
    STATE
        .get()
        .expect("`global::load` should be called without failing before global state is accessed")
}

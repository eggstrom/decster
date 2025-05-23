use anyhow::Result;

use crate::{cli, config, env::Env, state::State};

pub struct App {
    pub env: Env,
    pub state: State,
}

impl App {
    pub fn run() -> Result<()> {
        let env = Env::load()?;
        let state = State::load(&env)?;
        config::load(&env)?;
        let app = App { env, state };
        cli::run(app)
    }
}

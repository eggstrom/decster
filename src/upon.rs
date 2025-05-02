use std::{collections::HashMap, sync::OnceLock};

use anyhow::Result;
use thiserror::Error;
use toml::Value;
use upon::Engine;

static ENGINE: OnceLock<Engine> = OnceLock::new();

fn engine() -> &'static Engine<'static> {
    match ENGINE.get() {
        Some(engine) => engine,
        None => {
            let engine = Engine::new();
            ENGINE.get_or_init(|| engine)
        }
    }
}

/// Error type that wraps `upon::Error` to enable pretty printing in `anyhow`
/// context.
#[derive(Debug, Error)]
enum TemplateError {
    #[error("Couldn't compile template: {0:#}")]
    Compile(upon::Error),
    #[error("Couldn't render template: {0:#}")]
    Render(upon::Error),
}

pub fn render(template: &str, context: &HashMap<&str, &Value>) -> Result<String> {
    let engine = engine();
    Ok(engine
        .compile(template)
        .map_err(TemplateError::Compile)?
        .render(engine, context)
        .to_string()
        .map_err(TemplateError::Render)?)
}

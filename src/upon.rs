use std::{collections::HashMap, sync::OnceLock};

use anyhow::Result;
use derive_more::Display;
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
#[derive(Display, Debug, Error)]
enum TemplateError {
    #[display("Couldn't compile template: {_0:#}")]
    Compile(upon::Error),
    #[display("Couldn't render template: {_0:#}")]
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

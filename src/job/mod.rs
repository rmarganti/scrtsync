use crate::sources::Source;
use anyhow::Result;
use std::error;
use std::fmt;
use std::io;
use std::io::IsTerminal;

mod init;
mod sync;

pub trait Job {
    fn run(&self) -> Result<()>;
}

pub fn new_job(
    config: &crate::config::Config,
    from: Option<String>,
    to: Option<String>,
    preset: Option<String>,
) -> Result<Box<dyn Job>> {
    if preset == Some("init".to_string()) {
        return Ok(Box::new(init::InitJob {}));
    }

    let preset_cfg = preset.as_ref().and_then(|p| config.presets.get(p));

    let from = if io::stdin().is_terminal() {
        from
    } else {
        // Use stdin if piping in
        Some("std://".to_string())
    };

    let from = from // stdin (when piping) and `--from` take precedent
        .or_else(|| preset_cfg.map(|p| p.from.clone()))
        .map(|uri| <dyn Source>::new(&uri)) // Build the Source
        .unwrap_or_else(|| Err(NoSourceProvidedError::new("from").into()))?;

    let to = if io::stdout().is_terminal() {
        to
    } else {
        // Use stdin if piping out
        Some("std://".to_string())
    };

    let to = to // stdout (when piping) and `--to` take precedent
        .or_else(|| preset_cfg.map(|p| p.to.clone()))
        .map(|uri| <dyn Source>::new(&uri)) // Build the Source
        .unwrap_or_else(|| Err(NoSourceProvidedError::new("to").into()))?;

    Ok(Box::new(sync::SyncJob::new(from, to)))
}

#[derive(Debug, Clone)]
struct NoSourceProvidedError {
    field: &'static str,
}

impl NoSourceProvidedError {
    fn new(field: &'static str) -> Self {
        NoSourceProvidedError { field }
    }
}

impl fmt::Display for NoSourceProvidedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "unable to determine source, provide either `--{}` or a preset",
            self.field,
        )
    }
}

impl error::Error for NoSourceProvidedError {}

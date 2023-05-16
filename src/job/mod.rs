mod init;
mod sync;

use crate::sources::Source;
use atty;
use std::error;
use std::fmt;

pub trait Job {
    fn run(&self) -> crate::Result<()>;
}

pub fn new_job(
    config: &crate::config::Config,
    from: Option<String>,
    to: Option<String>,
    preset: Option<String>,
) -> crate::Result<Box<dyn Job>> {
    if preset == Some("init".to_string()) {
        return Ok(Box::new(init::InitJob {}));
    }

    let preset_cfg = preset.as_ref().map(|p| config.presets.get(p)).flatten();

    let from = if atty::is(atty::Stream::Stdin) {
        from
    } else {
        // Use stdin if piping in
        Some("std://".to_string())
    };

    let from = from // stdin (when piping) and `--from` take precedent
        .or(preset_cfg.map(|p| p.from.clone())) // Then see if a preset was specified
        .map(|uri| <dyn Source>::new(&uri)) // Build the Source
        .unwrap_or_else(|| {
            return Err(Box::new(NoSourceProvidedError::new("from")));
        })?;

    let to = if atty::is(atty::Stream::Stdout) {
        to
    } else {
        // Use stdin if piping out
        Some("std://".to_string())
    };

    let to = to // stdout (when piping) and `--to` take precedent
        .or(preset_cfg.map(|p| p.to.clone())) // Then see if a preset was specified
        .map(|uri| <dyn Source>::new(&uri)) // Build the Source
        .unwrap_or_else(|| {
            return Err(Box::new(NoSourceProvidedError::new("to")));
        })?;

    return Ok(Box::new(sync::SyncJob::new(from, to)));
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
            "unable to determine source, provide either `--{}` or a preset\n",
            self.field,
        )
    }
}

impl error::Error for NoSourceProvidedError {}

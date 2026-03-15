use crate::sources::{Source, SourceCreateError};
use anyhow::Result;
use std::io::IsTerminal;

mod diff;
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
    diff: bool,
) -> Result<Box<dyn Job>> {
    if preset == Some("init".to_string()) {
        return Ok(Box::new(init::InitJob {}));
    }

    let preset_cfg = preset.as_ref().and_then(|p| config.presets.get(p));

    if diff {
        // In diff mode we never read from stdin or write to stdout automatically.
        // Both `from` and `to` are required (enforced by Args::validate).
        let from_uri = from
            .or_else(|| preset_cfg.map(|p| p.from.clone()))
            .ok_or(SourceCreateError::NoSourceProvided { field: "from" })?;
        let from_source = <dyn Source>::new(&from_uri)?;

        let to_uri = to
            .or_else(|| preset_cfg.map(|p| p.to.clone()))
            .ok_or(SourceCreateError::NoSourceProvided { field: "to" })?;
        let to_source = <dyn Source>::new(&to_uri)?;

        return Ok(Box::new(diff::DiffJob::new(from_source, to_source, to_uri)));
    }

    let from = if std::io::stdin().is_terminal() {
        from
    } else {
        Some("std://".to_string())
    };

    let from = from
        .or_else(|| preset_cfg.map(|p| p.from.clone()))
        .ok_or(SourceCreateError::NoSourceProvided { field: "from" })?;
    let from = <dyn Source>::new(&from)?;

    let to = if std::io::stdout().is_terminal() {
        to
    } else {
        Some("std://".to_string())
    };

    let to = to
        .or_else(|| preset_cfg.map(|p| p.to.clone()))
        .ok_or(SourceCreateError::NoSourceProvided { field: "to" })?;
    let to = <dyn Source>::new(&to)?;

    Ok(Box::new(sync::SyncJob::new(from, to)))
}

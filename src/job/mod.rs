use crate::config::SyncArgs;
use crate::sources::{Source, SourceCreateError};
use anyhow::Result;
use std::io::IsTerminal;

mod diff;
pub mod init;
mod sync;

pub trait Job {
    fn run(&self) -> Result<()>;
}

pub fn build_sync_job(config: &crate::config::Config, args: SyncArgs) -> Result<Box<dyn Job>> {
    let preset_cfg = args.preset.as_ref().and_then(|p| config.presets.get(p));

    if args.diff {
        let from_uri = args
            .from
            .or_else(|| preset_cfg.map(|p| p.from.clone()))
            .ok_or(SourceCreateError::NoSourceProvided { field: "from" })?;
        let from_source = <dyn Source>::new(&from_uri)?;

        let to_uri = args
            .to
            .or_else(|| preset_cfg.map(|p| p.to.clone()))
            .ok_or(SourceCreateError::NoSourceProvided { field: "to" })?;
        let to_source = <dyn Source>::new(&to_uri)?;

        return Ok(Box::new(diff::DiffJob::new(from_source, to_source, to_uri)));
    }

    let from = if std::io::stdin().is_terminal() {
        args.from
    } else {
        Some("std://".to_string())
    };

    let from = from
        .or_else(|| preset_cfg.map(|p| p.from.clone()))
        .ok_or(SourceCreateError::NoSourceProvided { field: "from" })?;
    let from = <dyn Source>::new(&from)?;

    let to = if std::io::stdout().is_terminal() {
        args.to
    } else {
        Some("std://".to_string())
    };

    let to = to
        .or_else(|| preset_cfg.map(|p| p.to.clone()))
        .ok_or(SourceCreateError::NoSourceProvided { field: "to" })?;
    let to = <dyn Source>::new(&to)?;

    Ok(Box::new(sync::SyncJob::new(from, to)))
}

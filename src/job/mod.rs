use crate::sources::{Source, SourceCreateError};
use anyhow::Result;
use std::io::IsTerminal;

mod diff;
mod init;
mod sync;

type NamedSource = (String, Box<dyn Source>);

pub trait Job {
    fn run(&self) -> Result<()>;
}

pub fn new_job(
    config: &crate::config::Config,
    from: Vec<String>,
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
        let from_uris = resolve_from_uris(from, preset_cfg, false)?;
        let from_sources = build_sources(from_uris)?;

        let to_uri = to
            .or_else(|| preset_cfg.map(|p| p.to.clone()))
            .ok_or(SourceCreateError::NoSourceProvided { field: "to" })?;
        let to_source = <dyn Source>::new(&to_uri)?;

        return Ok(Box::new(diff::DiffJob::new(
            from_sources,
            to_source,
            to_uri,
        )));
    }

    let from_uris = resolve_from_uris(from, preset_cfg, !std::io::stdin().is_terminal())?;
    let from_sources = build_sources(from_uris)?;

    let to = if std::io::stdout().is_terminal() {
        to
    } else {
        Some("std://".to_string())
    };

    let to = to
        .or_else(|| preset_cfg.map(|p| p.to.clone()))
        .ok_or(SourceCreateError::NoSourceProvided { field: "to" })?;
    let to = <dyn Source>::new(&to)?;

    Ok(Box::new(sync::SyncJob::new(from_sources, to)))
}

fn resolve_from_uris(
    cli_from: Vec<String>,
    preset_cfg: Option<&crate::config::PresetConfig>,
    use_stdin: bool,
) -> Result<Vec<String>, SourceCreateError> {
    if use_stdin && cli_from.is_empty() {
        return Ok(vec!["std://".to_string()]);
    }

    if !cli_from.is_empty() {
        return Ok(cli_from);
    }

    let preset_from = preset_cfg
        .map(|p| p.from.to_vec())
        .ok_or(SourceCreateError::NoSourceProvided { field: "from" })?;

    if preset_from.is_empty() {
        return Err(SourceCreateError::NoSourceProvided { field: "from" });
    }

    Ok(preset_from)
}

fn build_sources(uris: Vec<String>) -> Result<Vec<NamedSource>, SourceCreateError> {
    uris.into_iter()
        .map(|uri| {
            let source = <dyn Source>::new(&uri)?;
            Ok((uri, source))
        })
        .collect()
}

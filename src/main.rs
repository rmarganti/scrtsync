use crate::config::{Args, Config};
use anyhow::{Context, Result};
use clap::Parser;

mod config;
mod job;
mod secrets;
mod sources;

fn main() -> Result<()> {
    let args = Args::parse();
    let cfg = Config::from_file(&args.config)
        .with_context(|| format!("failed to load config from '{}'", args.config))?;

    args.validate(&cfg)?;

    let job = job::new_job(&cfg, args.from, args.to, args.preset, args.diff)
        .context("could not build job")?;

    job.run()?;

    Ok(())
}

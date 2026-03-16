use crate::config::{Args, Command, Config};
use crate::job::Job;
use anyhow::{Context, Result};
use clap::Parser;

mod config;
mod job;
mod secrets;
mod sources;

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(Command::Init) = args.command {
        return job::init::InitJob::new().run();
    }

    let cfg = Config::from_file(&args.config)
        .with_context(|| format!("failed to load config from '{}'", args.config))?;

    let sync_args = args.sync_args;
    sync_args.validate(&cfg)?;

    let job = job::build_sync_job(&cfg, sync_args).context("could not build job")?;

    job.run()?;

    Ok(())
}

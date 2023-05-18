use crate::config::{Args, Config};
use anyhow::{Context, Result};
use clap::Parser;

mod config;
mod job;
mod secrets;
mod sources;

fn main() -> Result<()> {
    do_main()
}

fn do_main() -> Result<()> {
    let args = Args::parse();
    let cfg = Config::from_file(&args.config).with_context(|| "Could not build config object")?;

    let job = job::new_job(&cfg, args.from, args.to, args.preset)
        .with_context(|| "Could not build job")?;

    job.run()?;

    Ok(())
}

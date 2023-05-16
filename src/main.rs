use crate::config::{Args, Config};
use clap::Parser;

mod config;
mod job;
mod secrets;
mod sources;

fn main() {
    match do_main() {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
    }
}

fn do_main() -> Result<()> {
    let args = Args::parse();
    let cfg = Config::from_file(&args.config)?;

    let job = job::new_job(&cfg, args.from, args.to, args.preset)?;
    job.run()?;

    Ok(())
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

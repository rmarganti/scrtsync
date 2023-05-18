use anyhow::{Context, Result};
use std::io::Write;

use super::Job;

const DEFAULT_CONFIG: &[u8] = include_bytes!(".scrtsync.default.json");
const DEFAULT_FILENAME: &str = ".scrtsync.json";

/// This job is used to create a default config file.
pub struct InitJob {}

impl Job for InitJob {
    /// Write the default config contents to the default config file.
    fn run(&self) -> Result<()> {
        let path = std::path::Path::new(DEFAULT_FILENAME);
        if path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("{} already exists", DEFAULT_FILENAME),
            )
            .into());
        }

        let mut file =
            std::fs::File::create(path).with_context(|| "Unable to create default config")?;

        file.write_all(DEFAULT_CONFIG)
            .with_context(|| "Unable to write default config")?;

        Ok(())
    }
}

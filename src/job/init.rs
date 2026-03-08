use std::io::Write;

use super::Job;

const DEFAULT_CONFIG: &[u8] = include_bytes!(".scrtsync.default.json");
const DEFAULT_FILENAME: &str = ".scrtsync.json";

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("{0} already exists")]
    AlreadyExists(String),

    #[error("unable to create default config")]
    CreateFile(#[source] std::io::Error),

    #[error("unable to write default config")]
    WriteFile(#[source] std::io::Error),
}

/// This job is used to create a default config file.
pub struct InitJob {}

impl Job for InitJob {
    /// Write the default config contents to the default config file.
    fn run(&self) -> anyhow::Result<()> {
        let path = std::path::Path::new(DEFAULT_FILENAME);
        if path.exists() {
            return Err(InitError::AlreadyExists(DEFAULT_FILENAME.to_string()).into());
        }

        let mut file = std::fs::File::create(path).map_err(InitError::CreateFile)?;

        file.write_all(DEFAULT_CONFIG)
            .map_err(InitError::WriteFile)?;

        Ok(())
    }
}

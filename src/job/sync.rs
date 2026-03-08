use crate::sources::Source;
use anyhow::{Context, Result};

pub struct SyncJob {
    origin: Box<dyn Source>,
    target: Box<dyn Source>,
}

impl SyncJob {
    pub fn new(origin: Box<dyn Source>, target: Box<dyn Source>) -> Self {
        Self { origin, target }
    }
}

impl super::Job for SyncJob {
    /// Synchronize the secrets from an origin to a target
    fn run(&self) -> Result<()> {
        let secrets = self
            .origin
            .read_secrets()
            .context("unable to read secrets from source")?;

        self.target
            .write_secrets(&secrets)
            .context("unable to write secrets to target")?;

        Ok(())
    }
}

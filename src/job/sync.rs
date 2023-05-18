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
    /// Synchronise the secrets from an origin to a target
    fn run(&self) -> Result<()> {
        let secrets = self
            .origin
            .read_secrets()
            .with_context(|| "Unable to read secrets")?;

        self.target
            .write_secrets(&secrets)
            .with_context(|| "Unable to write secrets")?;

        Ok(())
    }
}

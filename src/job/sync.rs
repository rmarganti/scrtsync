use crate::sources::Source;

pub struct SyncJob {
    origin: Box<dyn Source>,
    target: Box<dyn Source>,
}

impl SyncJob {
    pub fn new(origin: Box<dyn Source>, target: Box<dyn Source>) -> Self {
        Self {
            origin,
            target,
        }
    }
}

impl super::Job for SyncJob {
    /// Synchronise the secrets from an origin to a target
    fn run(&self) -> crate::Result<()> {
        let secrets = self.origin.read_secrets()?;
        self.target.write_secrets(&secrets)?;

        Ok(())
    }
}

use super::NamedSource;
use crate::{secrets::Secrets, sources::Source};
use anyhow::{Context, Result};

pub struct SyncJob {
    origins: Vec<NamedSource>,
    target: Box<dyn Source>,
}

impl SyncJob {
    pub fn new(origins: Vec<NamedSource>, target: Box<dyn Source>) -> Self {
        Self { origins, target }
    }
}

impl super::Job for SyncJob {
    /// Synchronize merged secrets from one or more origins to a target.
    fn run(&self) -> Result<()> {
        let mut source_secrets = Vec::with_capacity(self.origins.len());

        for (uri, source) in &self.origins {
            let secrets = source
                .read_secrets()
                .with_context(|| format!("unable to read secrets from source '{uri}'"))?;
            source_secrets.push((uri.clone(), secrets));
        }

        let secrets =
            Secrets::merge_named(source_secrets).context("unable to merge source secrets")?;

        self.target
            .write_secrets(&secrets)
            .context("unable to write secrets to target")?;

        Ok(())
    }
}

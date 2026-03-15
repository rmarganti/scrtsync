use crate::secrets::Secrets;

pub struct StdInOutSource;

#[derive(Debug, thiserror::Error)]
pub enum StdInOutSourceError {
    #[error("unable to pipe secrets from stdin")]
    StdIn(#[source] crate::secrets::SecretsError),

    #[error("unable to pipe secrets to stdout")]
    StdOut(#[source] crate::secrets::SecretsError),
}

impl StdInOutSource {
    pub fn new() -> Self {
        StdInOutSource
    }
}

impl super::Source for StdInOutSource {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets, super::SourceSecretsError> {
        let secrets =
            Secrets::from_reader(&mut std::io::stdin()).map_err(StdInOutSourceError::StdIn)?;

        Ok(secrets)
    }

    fn write_secrets(
        &self,
        secrets: &crate::secrets::Secrets,
    ) -> Result<(), super::SourceSecretsError> {
        secrets
            .to_writer(&mut std::io::stdout())
            .map_err(StdInOutSourceError::StdOut)?;

        Ok(())
    }
}

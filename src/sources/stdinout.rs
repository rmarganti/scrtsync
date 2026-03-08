use crate::secrets::Secrets;
use anyhow::Result;

pub struct StdInOutSource;

impl StdInOutSource {
    pub fn new() -> Self {
        StdInOutSource
    }
}

impl super::Source for StdInOutSource {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets> {
        Ok(Secrets::from_reader(&mut std::io::stdin())?)
    }

    fn write_secrets(&self, secrets: &crate::secrets::Secrets) -> Result<()> {
        Ok(secrets.to_writer(&mut std::io::stdout())?)
    }
}

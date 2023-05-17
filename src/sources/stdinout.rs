use crate::secrets::Secrets;

pub struct StdInOutSource {}

impl StdInOutSource {
    pub fn new() -> crate::Result<Self> {
        Ok(StdInOutSource {})
    }
}

impl super::Source for StdInOutSource {
    fn read_secrets(&self) -> crate::Result<crate::secrets::Secrets> {
        Secrets::from_reader(&mut std::io::stdin())
    }

    fn write_secrets(&self, secrets: &crate::secrets::Secrets) -> crate::Result<()> {
        secrets.to_writer(&mut std::io::stdout())
    }
}

use anyhow::{anyhow, Context, Result};
use url::Url;

mod file;
mod k8s;
mod stdinout;
mod vault;

pub trait Source {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets>;
    fn write_secrets(&self, secrets: &crate::secrets::Secrets) -> Result<()>;
}

impl dyn Source {
    pub fn new(uri: &str) -> Result<Box<dyn Source>> {
        let url = Url::parse(uri).with_context(|| "Unable to parse source URL")?;

        let source: Box<dyn Source> = match url.scheme() {
            "file" => Box::new(
                file::FileSource::new(&url).with_context(|| "Could not build file source")?,
            ),
            "k8s" | "kubernetes" => Box::new(k8s::K8sSource::new(&url)?),
            "std" => Box::new(
                stdinout::StdInOutSource::new()
                    .with_context(|| "Could not build stdin/out source")?,
            ),
            "vault" => Box::new(
                vault::VaultSource::new(&url).with_context(|| "Could not build Vault source")?,
            ),
            _ => {
                return Err(anyhow!(format!(
                    "Unsupported source scheme: {}",
                    url.scheme()
                )));
            }
        };

        Ok(source)
    }
}

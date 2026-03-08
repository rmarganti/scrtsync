use anyhow::Result;
use url::Url;

mod file;
mod k8s;
mod stdinout;
mod vault;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("unsupported source scheme: {0}")]
    UnsupportedScheme(String),

    #[error("unable to parse source URL")]
    InvalidUrl(#[from] url::ParseError),

    #[error("unable to determine source, provide either `--{field}` or a preset")]
    NoSourceProvided { field: &'static str },

    #[error("could not build file source")]
    File(#[from] file::FileSourceError),

    #[error("could not build Kubernetes source")]
    K8s(#[from] k8s::K8sSourceError),

    #[error("could not build Vault source")]
    Vault(#[from] vault::VaultSourceError),
}

/// Source trait for reading/writing secrets. Methods return `anyhow::Result`
/// intentionally — runtime errors come from diverse backends (kube, HTTP, io)
/// and callers don't need to match on specific variants.
pub trait Source {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets>;
    fn write_secrets(&self, secrets: &crate::secrets::Secrets) -> Result<()>;
}

impl dyn Source {
    pub fn new(uri: &str) -> Result<Box<dyn Source>, SourceError> {
        let url = Url::parse(uri)?;

        let source: Box<dyn Source> = match url.scheme() {
            "file" => Box::new(file::FileSource::new(&url)?),
            "k8s" | "kubernetes" => Box::new(k8s::K8sSource::new(&url)?),
            "std" => Box::new(stdinout::StdInOutSource::new()),
            "vault" => Box::new(vault::VaultSource::new(&url)?),
            other => return Err(SourceError::UnsupportedScheme(other.to_string())),
        };

        Ok(source)
    }
}

use std::{error, fmt};
use url::Url;

mod file;
mod k8s;
mod stdinout;
mod vault;

pub trait Source {
    fn read_secrets(&self) -> crate::Result<crate::secrets::Secrets>;
    fn write_secrets(&self, secrets: &crate::secrets::Secrets) -> crate::Result<()>;
}

impl dyn Source {
    pub fn new(uri: &str) -> crate::Result<Box<dyn Source>> {
        let url = Url::parse(uri)?;

        let source: Box<dyn Source> = match url.scheme() {
            "file" => Box::new(file::FileSource::new(&url)?),
            "k8s" | "kubernetes" => Box::new(k8s::K8sSource::new(&url)?),
            "std" => Box::new(stdinout::StdInOutSource::new()?),
            "vault" => Box::new(vault::VaultSource::new(&url)?),
            _ => {
                return Err(Box::new(UnsupportedSourceError::new(
                    url.scheme().to_string(),
                )));
            }
        };

        Ok(source)
    }
}

#[derive(Debug, Clone)]
struct UnsupportedSourceError {
    scheme: String,
}

impl UnsupportedSourceError {
    fn new(scheme: String) -> Self {
        UnsupportedSourceError { scheme }
    }
}

impl fmt::Display for UnsupportedSourceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "invalid source scheme: `{}`, supported schemes: `file`",
            self.scheme,
        )
    }
}

impl error::Error for UnsupportedSourceError {}

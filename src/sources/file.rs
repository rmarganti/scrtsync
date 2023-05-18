use crate::secrets::Secrets;
use anyhow::{anyhow, Result};

pub struct FileSource {
    path: String,
}

impl FileSource {
    pub fn new(url: &url::Url) -> Result<Self> {
        let mut path = match url.host() {
            Some(host) => host.to_string(),
            None => return Err(anyhow!("Unable to parse file path")),
        };

        path.push_str(url.path());

        Ok(FileSource {
            path: path.trim_matches('/').to_string(),
        })
    }
}

impl super::Source for FileSource {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets> {
        eprintln!("Reading secrets from file at {}", self.path);
        let mut file = std::fs::File::open(&self.path)?;
        Secrets::from_reader(&mut file)
    }

    fn write_secrets(&self, secrets: &crate::secrets::Secrets) -> Result<()> {
        eprintln!("Writing secrets to file at {}", self.path);
        let mut file = std::fs::File::create(&self.path)?;
        secrets.to_writer(&mut file)
    }
}

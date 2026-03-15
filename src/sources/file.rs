use crate::secrets::Secrets;

#[derive(Debug, thiserror::Error)]
pub enum FileSourceError {
    #[error("unable to parse file path from URL")]
    InvalidPath,

    #[error("unable to open file for reading")]
    OpenFile(#[source] std::io::Error),

    #[error("unable to create file for writing")]
    CreateFile(#[source] std::io::Error),

    #[error("unable to parse secrets")]
    Parse(#[source] crate::secrets::SecretsError),

    #[error("unable to write secrets")]
    Write(#[source] crate::secrets::SecretsError),
}

pub struct FileSource {
    path: String,
}

impl FileSource {
    pub fn new(url: &url::Url) -> Result<Self, FileSourceError> {
        let mut path = match url.host() {
            Some(host) => host.to_string(),
            None => return Err(FileSourceError::InvalidPath),
        };

        path.push_str(url.path());

        Ok(FileSource {
            path: path.trim_matches('/').to_string(),
        })
    }
}

impl super::Source for FileSource {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets, super::SourceSecretsError> {
        eprintln!("Reading secrets from file at {}", self.path);
        let mut file = std::fs::File::open(&self.path).map_err(FileSourceError::OpenFile)?;
        let secrets = Secrets::from_reader(&mut file).map_err(FileSourceError::Parse)?;

        Ok(secrets)
    }

    fn write_secrets(
        &self,
        secrets: &crate::secrets::Secrets,
    ) -> Result<(), super::SourceSecretsError> {
        eprintln!("Writing secrets to file at {}", self.path);
        let mut file = std::fs::File::create(&self.path).map_err(FileSourceError::CreateFile)?;

        secrets
            .to_writer(&mut file)
            .map_err(FileSourceError::Write)?;

        Ok(())
    }
}

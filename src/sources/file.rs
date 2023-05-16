use crate::secrets::Secrets;

pub struct FileSource {
    path: String,
}

impl FileSource {
    pub fn new(url: &url::Url) -> crate::Result<Self> {
        let mut path = url.host().unwrap().to_string();
        path.push_str(url.path());

        Ok(FileSource {
            path: path.trim_matches('/').to_string(),
        })
    }
}

impl super::Source for FileSource {
    fn read_secrets(&self) -> crate::Result<crate::secrets::Secrets> {
        let mut file = std::fs::File::open(&self.path)?;
        Secrets::from_reader(&mut file)
    }

    fn write_secrets(&self, secrets: &crate::secrets::Secrets) -> crate::Result<()> {
        let mut file = std::fs::File::create(&self.path)?;
        secrets.to_env(&mut file)
    }
}

use crate::secrets::Secrets;
use anyhow::Result;

pub struct K8sSource {
    _context: String,
    _secret_name: String,
}

impl K8sSource {
    pub fn new(url: &url::Url) -> Result<Self> {
        Ok(K8sSource {
            _context: url.host().unwrap().to_string(),
            _secret_name: url.path().to_string(),
        })
    }
}

impl super::Source for K8sSource {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets> {
        Ok(Secrets::new())
    }

    fn write_secrets(&self, _secrets: &crate::secrets::Secrets) -> Result<()> {
        Ok(())
    }
}

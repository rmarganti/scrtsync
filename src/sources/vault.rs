use crate::secrets::Secrets;

pub struct VaultSource {
    _mounth_path: String,
    _secret_path: String,
}

impl VaultSource {
    pub fn new(url: &url::Url) -> crate::Result<Self> {
        Ok(VaultSource {
            _mounth_path: url.host().unwrap().to_string(),
            _secret_path: url.path().to_string(),
        })
    }
}

impl super::Source for VaultSource {
    fn read_secrets(&self) -> crate::Result<crate::secrets::Secrets> {
        Ok(Secrets::new())
    }

    fn write_secrets(&self, _secrets: &crate::secrets::Secrets) -> crate::Result<()> {
        Ok(())
    }
}

use crate::secrets::Secrets;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use ureq::{Agent, AgentBuilder};

#[derive(Debug, thiserror::Error)]
pub enum VaultSourceError {
    #[error("Vault URL missing host")]
    MissingHost,

    #[error("Vault URL host cannot be empty")]
    EmptyHost,

    #[error("VAULT_ADDR environment variable not set")]
    MissingVaultAddr,

    #[error("could not determine home directory for ~/.vault-token")]
    NoHomeDir,

    #[error("unable to read token from {path}")]
    ReadToken {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("unable to decode payload")]
    Decode(#[source] serde_json::Error),

    #[error("unable to encode payload")]
    Encode(#[source] serde_json::Error),

    #[error("network error")]
    Network(Box<ureq::Error>),
}

impl From<ureq::Error> for VaultSourceError {
    fn from(e: ureq::Error) -> Self {
        Self::Network(Box::new(e))
    }
}

pub struct VaultSource {
    client: Agent,
    token: String,
    mount_path: String,
    secret_path: String,
}

impl VaultSource {
    pub fn new(url: &url::Url) -> Result<Self, VaultSourceError> {
        let host = url.host().ok_or(VaultSourceError::MissingHost)?;
        let mount_path = host.to_string();

        if mount_path.is_empty() {
            return Err(VaultSourceError::EmptyHost);
        }

        Ok(VaultSource {
            mount_path,
            secret_path: url.path().to_string(),
            client: AgentBuilder::new()
                .timeout_read(std::time::Duration::from_secs(5))
                .timeout_write(std::time::Duration::from_secs(5))
                .build(),
            token: find_token()?,
        })
    }

    // Generate the full URL to read and write secrets.
    fn url(&self) -> Result<String, VaultSourceError> {
        let addr = env::var("VAULT_ADDR").map_err(|_| VaultSourceError::MissingVaultAddr)?;
        Ok(format!(
            "{}/v1/{}{}",
            addr, self.mount_path, self.secret_path
        ))
    }
}

impl super::Source for VaultSource {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets, super::SourceSecretsError> {
        let url = self.url()?;
        eprintln!("Reading secrets from Vault at {url}");

        let body = self
            .client
            .get(&url)
            .set("Content-Type", "application/json")
            .set("X-Vault-Token", &self.token)
            .call()
            .map_err(VaultSourceError::from)?
            .into_reader();

        let body: SecretResponse =
            serde_json::from_reader(body).map_err(VaultSourceError::Decode)?;

        let secrets: Secrets = body.data.into();

        Ok(secrets)
    }

    fn write_secrets(
        &self,
        secrets: &crate::secrets::Secrets,
    ) -> Result<(), super::SourceSecretsError> {
        let url = self.url()?;
        eprintln!("Writing secrets to Vault at {url}");

        let body = serde_json::to_string(&secrets.content).map_err(VaultSourceError::Encode)?;

        self.client
            .put(&url)
            .set("X-Vault-Token", &self.token)
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(VaultSourceError::from)?;

        Ok(())
    }
}

// Prioritize the VAULT_TOKEN environment variable. But fall back to reading from ~/.vault-token
fn find_token() -> Result<String, VaultSourceError> {
    if let Ok(token) = env::var("VAULT_TOKEN") {
        return Ok(token);
    }

    let home_dir = dirs::home_dir().ok_or(VaultSourceError::NoHomeDir)?;
    let token_path = home_dir.join(".vault-token");
    let token =
        std::fs::read_to_string(&token_path).map_err(|source| VaultSourceError::ReadToken {
            path: token_path.display().to_string(),
            source,
        })?;

    Ok(token.trim().to_string())
}

/// The shape of the response when fetching Secrets.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretResponse {
    #[serde(rename = "request_id")]
    pub request_id: String,
    #[serde(rename = "lease_id")]
    pub lease_id: String,
    pub renewable: bool,
    #[serde(rename = "lease_duration")]
    pub lease_duration: i64,
    pub data: BTreeMap<String, String>,
    #[serde(rename = "wrap_info")]
    pub wrap_info: Value,
    pub warnings: Value,
    pub auth: Value,
}

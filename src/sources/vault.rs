use crate::secrets::Secrets;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use ureq::{Agent, AgentBuilder};

pub struct VaultSource {
    client: Agent,
    token: String,
    mount_path: String,
    secret_path: String,
}

impl VaultSource {
    pub fn new(url: &url::Url) -> Result<Self> {
        let host = url
            .host()
            .ok_or_else(|| anyhow!("Vault URL missing host"))?;

        let mount_path = host.to_string();

        // Validate that the mount path is not empty after conversion
        if mount_path.is_empty() {
            return Err(anyhow!("Vault URL host cannot be empty"));
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
    fn url(&self) -> Result<String> {
        Ok(format!(
            "{}/v1/{}{}",
            env::var("VAULT_ADDR").with_context(|| "VAULT_ADDR environment variable not set")?,
            self.mount_path,
            self.secret_path
        ))
    }
}

impl super::Source for VaultSource {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets> {
        let url = self.url()?;
        eprintln!("Reading secrets from Vault at {url}");

        let body = self
            .client
            .get(&url)
            .set("Content-Type", "application/json")
            .set("X-Vault-Token", &self.token)
            .call()?
            .into_reader();

        let body: SecretResponse = serde_json::from_reader(body)
            .with_context(|| "Unable to parse Vault server response")?;
        let secrets: Secrets = body.data.into();

        Ok(secrets)
    }

    fn write_secrets(&self, secrets: &crate::secrets::Secrets) -> Result<()> {
        let url = self.url()?;
        eprintln!("Writing secrets to Vault at {url}");

        let body = serde_json::to_string(&secrets.content)
            .with_context(|| "Unable to encode server request")?;

        self.client
            .put(&url)
            .set("X-Vault-Token", &self.token)
            .set("Content-Type", "application/json")
            .send_string(&body)?;

        Ok(())
    }
}

// Prioritize the VAULT_TOKEN environment variable. But fall back to reading from ~/.vault-token
fn find_token() -> Result<String> {
    if let Ok(token) = env::var("VAULT_TOKEN") {
        return Ok(token);
    }

    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not determine home directory for ~/.vault-token"))?;
    let token_path = home_dir.join(".vault-token");
    let token = std::fs::read_to_string(&token_path)
        .with_context(|| format!("Unable to read token from {}", token_path.display()))?;

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

use crate::secrets::Secrets;
use anyhow::{anyhow, Context, Result};
use dirs;
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
        Ok(VaultSource {
            mount_path: url.host().unwrap().to_string(),
            secret_path: url.path().to_string(),
            client: AgentBuilder::new()
                .timeout_read(std::time::Duration::from_secs(5))
                .timeout_write(std::time::Duration::from_secs(5))
                .build(),
            token: find_token()?,
        })
    }

    // Generate the full URL to read and write secrets.
    fn url(&self) -> String {
        format!(
            "{}/v1/{}{}",
            env::var("VAULT_ADDR").unwrap(),
            self.mount_path,
            self.secret_path
        )
    }
}

impl super::Source for VaultSource {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets> {
        eprintln!("Reading secrets from Vault at {}", self.url());

        let body = self
            .client
            .get(&self.url())
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
        eprintln!("Writing secrets to Vault at {}", self.url());

        let body = serde_json::to_string(&SecretRequest {
            data: secrets.content.clone(),
        })
        .with_context(|| "Unable to encode server request")?;

        self.client
            .put(&self.url())
            .set("X-Vault-Token", &self.token)
            .set("Content-Type", "application/json")
            .send_string(&body)?;

        Ok(())
    }
}

// Prioritize the VAULT_TOKEN environment variable. But fall back to reading from ~/.vault-token
fn find_token() -> Result<String> {
    env::var("VAULT_TOKEN")
        .or_else(|_| {
            let mut path = dirs::home_dir().unwrap();
            path.push(".vault-token");
            std::fs::read_to_string(path)
        })
        .or_else(|_| {
            return Err(anyhow!(
                "Unable to find token in $VAULT_TOKEN or ~/.vault-token"
            ));
        })
}

/// The shape of the response when fetching Secrets.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// The shape of the request when storing Secrets.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretRequest {
    pub data: BTreeMap<String, String>,
}

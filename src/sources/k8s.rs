use crate::secrets::Secrets;
use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Secret as K8sSecret;
use kube::{
    api::{ObjectMeta, PostParams},
    config::KubeConfigOptions,
    Api,
};
use tokio::runtime::Runtime;

pub struct K8sSource {
    api: Api<K8sSecret>,
    runtime: Runtime,
    secret_name: String,
}

impl K8sSource {
    pub fn new(url: &url::Url) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let host = url
            .host()
            .ok_or_else(|| anyhow::anyhow!("URL missing host for Kubernetes context"))?;

        let context = host.to_string();

        // Validate that the context is not empty after conversion
        if context.is_empty() {
            return Err(anyhow::anyhow!("Kubernetes context cannot be empty"));
        }

        let api = runtime.block_on(create_k8s_secrets_api(context))?;

        Ok(K8sSource {
            api,
            secret_name: url.path().trim_matches('/').to_string(),
            runtime,
        })
    }
}

impl super::Source for K8sSource {
    fn read_secrets(&self) -> Result<crate::secrets::Secrets> {
        eprintln!("Reading secrets from k8s secret {}", self.secret_name);

        let body = self
            .runtime
            .block_on(self.api.get(self.secret_name.as_str()))?;

        let data = body.data.ok_or_else(|| anyhow!("Secret data not found"))?;

        Secrets::try_from(data)
    }

    fn write_secrets(&self, secrets: &crate::secrets::Secrets) -> Result<()> {
        eprintln!("Writing secrets to k8s secret {}", self.secret_name);

        self.runtime.block_on(create_or_update_secrets(
            &self.api,
            &self.secret_name,
            secrets,
        ))?;

        Ok(())
    }
}

async fn create_k8s_secrets_api(context: String) -> Result<Api<K8sSecret>> {
    let options = KubeConfigOptions {
        context: Some(context),
        ..KubeConfigOptions::default()
    };

    let config = kube::Config::from_kubeconfig(&options).await?;
    let client = kube::Client::try_from(config)?;
    let api: Api<K8sSecret> = Api::default_namespaced(client);

    Ok(api)
}

async fn create_or_update_secrets(
    api: &Api<K8sSecret>,
    secret_name: &str,
    secrets: &crate::secrets::Secrets,
) -> Result<()> {
    let payload = K8sSecret {
        metadata: ObjectMeta {
            name: Some(secret_name.to_string()),
            ..ObjectMeta::default()
        },
        string_data: Some(secrets.content.clone()),
        ..K8sSecret::default()
    };

    let existing_secret = api.get_opt(secret_name).await?;

    match existing_secret {
        Some(_) => {
            api.replace(secret_name, &PostParams::default(), &payload)
                .await?
        }
        None => api.create(&PostParams::default(), &payload).await?,
    };

    Ok(())
}

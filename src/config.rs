use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::{collections::HashMap, fs, io, path};

const DEFAULT_CONFIG: &str = ".scrtsync.json";

/// Synchronize secrets between different sources
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]

pub struct Args {
    /// Config file to use for presets
    #[arg(short, long, default_value_t = String::from(DEFAULT_CONFIG))]
    pub config: String,

    /// From where to pull secrets
    #[arg(short, long)]
    pub from: Option<String>,

    /// To where to output secrets
    #[arg(short, long)]
    pub to: Option<String>,

    /// An optional preset defined in a config file
    pub preset: Option<String>,
}

impl Args {
    pub fn validate(&self, config: &Config) -> Result<()> {
        // If preset is provided, ensure it exists
        if let Some(ref preset_name) = self.preset {
            if preset_name != "init" && !config.presets.contains_key(preset_name) {
                return Err(anyhow::anyhow!(
                    "Preset '{}' not found in config file",
                    preset_name
                ));
            }
        }

        // If no preset and no from/to args, that's an error
        if self.preset.is_none() && self.from.is_none() && self.to.is_none() {
            return Err(anyhow::anyhow!(
                "Must provide either a preset or both --from and --to arguments"
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub presets: HashMap<String, PresetConfig>,
}

#[derive(Debug, Deserialize)]
pub struct PresetConfig {
    pub from: String,
    pub to: String,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config> {
        let exists = path::Path::new(path).exists();

        // If the file does not exist, return default config
        if !exists && path == DEFAULT_CONFIG {
            return Ok(Config::default());
        }

        let file = fs::File::open(path).with_context(|| "Could not open config")?;
        let reader = io::BufReader::new(file);

        let cfg: Config =
            serde_json::from_reader(reader).with_context(|| "Could not parse config file")?;

        cfg.validate()
            .with_context(|| "Configuration validation failed")?;

        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        for (name, preset) in &self.presets {
            // Validate that source URLs are parseable
            url::Url::parse(&preset.from)
                .with_context(|| format!("Invalid 'from' URL in preset '{}'", name))?;
            url::Url::parse(&preset.to)
                .with_context(|| format!("Invalid 'to' URL in preset '{}'", name))?;
        }
        Ok(())
    }
}

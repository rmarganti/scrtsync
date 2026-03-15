use clap::Parser;
use serde::Deserialize;
use std::{collections::HashMap, fs, io, path};

const DEFAULT_CONFIG: &str = ".scrtsync.json";

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not open config file '{path}'")]
    OpenFile {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("could not parse config file")]
    Parse(#[source] serde_json::Error),

    #[error("invalid `{field}` URL in preset '{preset}'")]
    InvalidPresetUrl {
        preset: String,
        field: String,
        #[source]
        source: url::ParseError,
    },

    #[error("preset '{0}' not found in config file")]
    PresetNotFound(String),

    #[error("must provide either a preset or both --from and --to arguments")]
    MissingArguments,
}

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

    /// Show a diff between --from and --to without writing any secrets
    #[arg(short = 'd', long)]
    pub diff: bool,

    /// An optional preset defined in a config file
    pub preset: Option<String>,
}

impl Args {
    pub fn validate(&self, config: &Config) -> Result<(), ConfigError> {
        // If preset is provided, ensure it exists
        if let Some(ref preset_name) = self.preset {
            if preset_name != "init" && !config.presets.contains_key(preset_name) {
                return Err(ConfigError::PresetNotFound(preset_name.clone()));
            }
        }

        if self.diff {
            // Diff mode: need both sides (preset supplies both, or both --from and --to)
            let has_from = self.preset.is_some() || self.from.is_some();
            let has_to = self.preset.is_some() || self.to.is_some();

            if !has_from || !has_to {
                return Err(ConfigError::MissingArguments);
            }
        } else {
            // Sync mode: need both sides
            if self.preset.is_none() && (self.from.is_none() || self.to.is_none()) {
                return Err(ConfigError::MissingArguments);
            }
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
    pub fn from_file(path: &str) -> Result<Config, ConfigError> {
        let exists = path::Path::new(path).exists();

        // If the file does not exist, return default config
        if !exists && path == DEFAULT_CONFIG {
            return Ok(Config::default());
        }

        let file = fs::File::open(path).map_err(|source| ConfigError::OpenFile {
            path: path.to_string(),
            source,
        })?;
        let reader = io::BufReader::new(file);

        let cfg: Config = serde_json::from_reader(reader).map_err(ConfigError::Parse)?;

        cfg.validate()?;

        Ok(cfg)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        for (name, preset) in &self.presets {
            url::Url::parse(&preset.from).map_err(|source| ConfigError::InvalidPresetUrl {
                preset: name.clone(),
                field: "from".to_string(),
                source,
            })?;
            url::Url::parse(&preset.to).map_err(|source| ConfigError::InvalidPresetUrl {
                preset: name.clone(),
                field: "to".to_string(),
                source,
            })?;
        }
        Ok(())
    }
}

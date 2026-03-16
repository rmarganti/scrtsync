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

    #[error("--target is only valid when the source is a preset name, not a raw URI")]
    TargetRequiresPreset,

    #[error("must provide either a preset or both --from and --to arguments")]
    MissingArguments,

    #[error("--target is required when editing a preset")]
    EditMissingTarget,
}

/// Synchronize secrets between different sources
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, args_conflicts_with_subcommands = true)]
pub struct Args {
    /// Config file to use for presets
    #[arg(short, long, default_value_t = String::from(DEFAULT_CONFIG), global = true)]
    pub config: String,

    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub sync_args: SyncArgs,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Initialize a new config file
    Init,

    /// Edit secrets in-place for a single source
    Edit(EditArgs),
}

#[derive(clap::Args, Debug)]
pub struct EditArgs {
    /// Source URI or preset name
    pub source: String,

    /// When using a preset, which side to edit (from or to)
    #[arg(long)]
    pub target: Option<EditTarget>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum EditTarget {
    From,
    To,
}

impl EditArgs {
    /// Resolve the source URI. If `source` is a known preset name, `--target`
    /// is required to select which side. If it is a raw URI, `--target` must
    /// not be provided.
    pub fn resolve_uri(&self, config: &Config) -> Result<String, ConfigError> {
        if let Some(preset) = config.presets.get(&self.source) {
            let target = self.target.as_ref().ok_or(ConfigError::EditMissingTarget)?;
            Ok(match target {
                EditTarget::From => preset.from.clone(),
                EditTarget::To => preset.to.clone(),
            })
        } else if self.target.is_some() {
            // `--target` only makes sense when `source` is a preset name, not a raw URI
            Err(ConfigError::TargetRequiresPreset)
        } else {
            Ok(self.source.clone())
        }
    }
}

#[derive(clap::Args, Debug)]
pub struct SyncArgs {
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

impl SyncArgs {
    pub fn validate(&self, config: &Config) -> Result<(), ConfigError> {
        // If using a preset, ensure it exists.
        if let Some(ref preset_name) = self.preset {
            if !config.presets.contains_key(preset_name) {
                return Err(ConfigError::PresetNotFound(preset_name.clone()));
            }
        }

        // If not using a preset, both --to and --from are required.
        if self.preset.is_none() && (self.from.is_none() || self.to.is_none()) {
            return Err(ConfigError::MissingArguments);
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

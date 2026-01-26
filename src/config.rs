use std::fs::File;

use color_eyre::eyre::OptionExt;
use serde::{Deserialize, Serialize};

use crate::internal_prelude::*;

/// Global config of all rules that should be applied.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub artist_rewrites: Vec<Rewrite>,
    #[serde(default)]
    pub albumartist_rewrites: Vec<Rewrite>,
}

/// A rewrite condition that takes a list of expressions
///
/// If one of the expressions matches, the `single`, `multi` properties are set for the property in
/// question.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Rewrite {
    pub expressions: Vec<String>,
    #[serde(default)]
    pub single: Option<String>,
    #[serde(default)]
    pub multi: Option<Vec<String>>,
}

impl Config {
    /// Try to read existing config files.
    ///
    /// If none is found, a new one is created.
    pub fn read() -> Result<Config> {
        info!("Parsing config file");

        let config_dir = dirs::config_dir().ok_or_eyre("Couldn't determine config directory")?;
        let path = config_dir.join("beets").join("rename.yml");
        info!("Checking path: {path:?}");

        // Check if the file exists and parse it.
        if !path.exists() || !path.is_file() {
            info!("No config file found. Use and write default config.");

            let file = File::create(&path)
                .wrap_err_with(|| format!("Error creating config file at {path:?}"))?;

            let config = Config::default();
            serde_yaml::to_writer(file, &config)
                .wrap_err_with(|| format!("Error serializing to file at {path:?}"))?;

            // Return a default configuration if we couldn't find a file.
            return Ok(config);
        }

        info!("Found config file at: {path:?}");

        let file =
            File::open(&path).wrap_err_with(|| format!("Error opening config file at {path:?}"))?;
        let config = serde_yaml::from_reader(file)
            .wrap_err_with(|| format!("Error deserializing file at {path:?}"))?;
        Ok(config)
    }
}

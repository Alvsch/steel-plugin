use std::{
    collections::HashMap,
    fs,
    ops::{Deref, DerefMut},
    path::Path,
};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginSection {
    pub name: String,
    pub description: String,
    pub version: Version,
    pub api_version: VersionReq,
    pub authors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub plugin: PluginSection,
    #[serde(default)]
    pub exports: HashMap<String, String>,
    #[serde(default)]
    pub dependencies: HashMap<String, VersionReq>,
}

impl Config {
    pub fn from_root(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config = fs::read_to_string(path.as_ref().join("config.toml"))?;
        Ok(toml::from_str(&config)?)
    }
}

impl Deref for Config {
    type Target = PluginSection;

    fn deref(&self) -> &Self::Target {
        &self.plugin
    }
}

impl DerefMut for Config {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.plugin
    }
}

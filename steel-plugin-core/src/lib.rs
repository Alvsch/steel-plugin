use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const STEEL_API_VERSION: u32 = 0;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub depends: Vec<String>,
    pub api_version: u32,
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl PluginMeta {
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).expect("failed to serialize")
    }
}

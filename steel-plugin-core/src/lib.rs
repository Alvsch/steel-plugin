use std::path::PathBuf;

use semver::Version;
use serde::{Deserialize, Serialize};

pub const STEEL_API_VERSION: Version = Version::new(0, 2, 0);

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub version: Version,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends: Vec<String>,
    pub api_version: Version,
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl PluginMeta {
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        rmp_serde::to_vec_named(self).expect("failed to serialize")
    }
}

pub type TopicId = u32;

#[must_use]
pub const fn fnv1a_32(bytes: &[u8]) -> TopicId {
    let mut hash: u32 = 0x811C_9DC5;
    let mut i = 0;

    while i < bytes.len() {
        hash ^= bytes[i] as u32;
        hash = hash.wrapping_mul(0x0100_0193);
        i += 1;
    }

    hash
}

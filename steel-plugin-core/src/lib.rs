use serde::{Deserialize, Serialize};

pub const STEEL_API_VERSION: u32 = 0;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub depends: Vec<String>,
}

impl PluginMeta {
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).expect("failed to serialize")
    }
}

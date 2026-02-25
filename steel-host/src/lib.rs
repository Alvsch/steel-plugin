use std::{fmt::Debug, path::PathBuf};

use serde::Deserialize;
use wasmparser::{Parser, Payload};

pub use crate::exports::PluginExports;
pub use crate::loader::{PluginHostData, PluginLoader};
pub use crate::manager::PluginManager;

mod exports;
mod instance;
mod loader;
mod manager;
mod topological_sort;

fn read_custom_section<'a>(bytes: &'a [u8], name: &str) -> wasmparser::Result<Option<&'a [u8]>> {
    for payload in Parser::new(0).parse_all(bytes) {
        match payload? {
            Payload::CustomSection(reader) if reader.name() == name => {
                return Ok(Some(reader.data()));
            }
            _ => {}
        }
    }
    Ok(None)
}

#[derive(Debug, Deserialize, Clone)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub depends: Box<[String]>,
    #[serde(skip)]
    pub file_path: PathBuf,
}

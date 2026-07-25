use std::{collections::HashMap, fs, io, path::PathBuf};

use mlua::prelude::*;
use tempfile::TempDir;
use thiserror::Error;

use crate::{config::Config, stages::resolve::ResolvedPlugin};

pub struct CompiledPlugin {
    pub root: PathBuf,
    pub config: Config,
    pub file_table: HashMap<String, PathBuf>,
    pub init_bytecode: Vec<u8>,
    pub(crate) extracted: Option<TempDir>,
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("plugin '{0}' has no init.lua/init.luau/init.luac entry file")]
    MissingEntryFile(String),

    #[error("failed to read entry file for plugin '{plugin}': {source}")]
    Io {
        plugin: String,
        #[source]
        source: io::Error,
    },

    #[error("failed to compile plugin '{plugin}': {source}")]
    Compile {
        plugin: String,
        #[source]
        source: LuaError,
    },
}

pub struct PluginCompiler {
    compiler: LuaCompiler,
}

impl Default for PluginCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginCompiler {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            compiler: LuaCompiler::new(),
        }
    }

    fn compile_bytecode(&self, source: impl AsRef<[u8]>) -> Result<Vec<u8>, mlua::Error> {
        self.compiler.compile(source)
    }

    pub fn compile(&self, plugin: ResolvedPlugin) -> Result<CompiledPlugin, CompileError> {
        let entry_path = plugin
            .file_table
            .get("init")
            .ok_or_else(|| CompileError::MissingEntryFile(plugin.config.name.clone()))?;

        let source = fs::read(entry_path).map_err(|source| CompileError::Io {
            plugin: plugin.config.name.clone(),
            source,
        })?;

        let init_bytecode = if entry_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("luac"))
        {
            source
        } else {
            self.compile_bytecode(&source)
                .map_err(|source| CompileError::Compile {
                    plugin: plugin.config.name.clone(),
                    source,
                })?
        };

        Ok(CompiledPlugin {
            root: plugin.root,
            config: plugin.config,
            file_table: plugin.file_table,
            init_bytecode,
            extracted: plugin.extracted,
        })
    }
}

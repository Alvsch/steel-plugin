use std::{
    io,
    path::{Path, PathBuf},
};

use rmp_serde::decode;
use thiserror::Error;
use tokio::fs::{create_dir_all, read, read_dir};
use tracing::error;
use wasmparser::BinaryReaderError;
use wasmtime::{Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder, p1::WasiP1Ctx};

use crate::{PluginExports, PluginMeta, read_custom_section, topological_sort};

pub struct PluginHostData {
    pub wasi: WasiP1Ctx,
    pub plugin_meta: PluginMeta,
}

pub struct LoadedPlugin {
    pub enabled: bool,
    pub store: Store<PluginHostData>,
    pub instance: Instance,
    pub exports: PluginExports,
}

impl LoadedPlugin {
    #[must_use]
    pub fn metadata(&self) -> &PluginMeta {
        &self.store.data().plugin_meta
    }
}

#[derive(Debug, Error)]
pub enum PluginLoaderError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("binary reader: {0}")]
    BinaryReader(#[from] BinaryReaderError),
    #[error("missing plugin meta")]
    MissingPluginMeta,
    #[error("invalid plugin meta: {0}")]
    InvalidPluginMeta(decode::Error),
    #[error("wasmtime: {0}")]
    Wasmtime(#[from] wasmtime::Error),
    #[error("plugin export resolve: {0}")]
    PluginExportResolve(wasmtime::Error),
}

pub struct PluginLoader {
    engine: Engine,
    linker: Linker<PluginHostData>,
    data_dir: PathBuf,
}

impl PluginLoader {
    #[must_use]
    pub const fn new(engine: Engine, linker: Linker<PluginHostData>, data_dir: PathBuf) -> Self {
        Self {
            engine,
            linker,
            data_dir,
        }
    }

    /// Discover plugins in the specificed directory and return their `PluginMeta` in topological order.
    pub async fn discover_plugins(
        &self,
        plugin_dir: &Path,
    ) -> Result<Vec<PluginMeta>, PluginLoaderError> {
        let mut plugins = Vec::new();

        let mut dir = read_dir(plugin_dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            let file_type = entry.file_type().await?;
            let file_path = entry.path();

            if file_type.is_file() {
                if file_path
                    .extension()
                    .is_none_or(|x| x.to_str().is_none_or(|ext| ext != "wasm"))
                {
                    continue;
                }
                let bytes = read(&file_path).await?;
                let meta_section = read_custom_section(&bytes, "plugin_meta")?
                    .ok_or(PluginLoaderError::MissingPluginMeta)?;
                let mut plugin_meta: PluginMeta = rmp_serde::from_slice(meta_section)
                    .map_err(PluginLoaderError::InvalidPluginMeta)?;
                plugin_meta.file_path = file_path;
                plugins.push(plugin_meta);
            }
        }
        let (topology, invalid) = topological_sort::sort_plugins(plugins);
        if !invalid.is_empty() {
            error!("plugins with invalid dependencies: {:#?}", invalid);
        }

        Ok(topology)
    }

    pub async fn load_plugin(
        &self,
        plugin_meta: PluginMeta,
    ) -> Result<LoadedPlugin, PluginLoaderError> {
        // read
        let bytes = read(&plugin_meta.file_path).await?;

        // compile
        let precompiled = self.engine.precompile_module(&bytes)?;
        let module = unsafe { Module::deserialize(&self.engine, precompiled) }?;

        // config
        let host_path = self.data_dir.join(&*plugin_meta.name);
        create_dir_all(&host_path).await?;

        let wasi = WasiCtxBuilder::new()
            .preopened_dir(host_path, "/", DirPerms::all(), FilePerms::all())?
            .build_p1();

        let mut store = Store::new(&self.engine, PluginHostData { wasi, plugin_meta });

        // init
        let instance = self.linker.instantiate_async(&mut store, &module).await?;
        let exports = PluginExports::resolve(&instance, &mut store)
            .map_err(PluginLoaderError::PluginExportResolve)?;

        Ok(LoadedPlugin {
            enabled: false,
            store,
            instance,
            exports,
        })
    }
}

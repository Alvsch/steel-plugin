use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use rmp_serde::decode;
use thiserror::Error;
use tokio::fs::{create_dir_all, read, read_dir};
use tracing::error;
use wasmparser::BinaryReaderError;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder, p1::WasiP1Ctx};

use crate::{
    EventRegistry, PluginExports, PluginMeta,
    instance::{PluginInstance, PluginStatus},
    read_custom_section, topological_sort,
};

pub struct PluginHostData {
    pub name: String,
    pub wasi: WasiP1Ctx,
    pub registry: Arc<EventRegistry>,
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
    registry: Arc<EventRegistry>,
}

impl PluginLoader {
    #[must_use]
    pub const fn new(
        engine: Engine,
        linker: Linker<PluginHostData>,
        data_dir: PathBuf,
        registry: Arc<EventRegistry>,
    ) -> Self {
        Self {
            engine,
            linker,
            data_dir,
            registry,
        }
    }

    /// Discover plugins in the specified directory and return their `PluginMeta` in topological order.
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
                plugin_meta.file_path = file_path.canonicalize()?;
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
    ) -> Result<PluginInstance, PluginLoaderError> {
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

        let mut store = Store::try_new(
            &self.engine,
            PluginHostData {
                name: plugin_meta.name.clone(),
                wasi,
                registry: self.registry.clone(),
            },
        )?;

        // init
        let instance = self.linker.instantiate_async(&mut store, &module).await?;
        let exports = PluginExports::resolve(&instance, &mut store)
            .map_err(PluginLoaderError::PluginExportResolve)?;

        Ok(PluginInstance {
            meta: plugin_meta,
            status: PluginStatus::Disabled,
            exports,
            memory: instance.get_memory(&mut store, "memory").unwrap(),
            store,
        })
    }
}

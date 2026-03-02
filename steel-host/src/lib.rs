pub use crate::event_registry::EventRegistry;
pub use crate::exports::PluginExports;
pub use crate::manager::PluginManager;
pub use instance::PluginInstance;
pub use meta::PluginMeta;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{create_dir_all, read, read_dir};
use tokio::sync::{Mutex, RwLock};
use tracing::error;
pub use utils::register_default_events;

use crate::error::PluginLoaderError;
use crate::linker::configure_all;
use crate::plugin::{PluginState, PluginStore};
use crate::rpc::HostRpc;
use crate::utils::read_custom_section;
use crate::utils::sorting::sort_plugins;
pub use wasmtime;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

pub mod error;
mod event_registry;
mod exports;
mod instance;
pub mod linker;
mod manager;
mod meta;
pub mod plugin;
pub mod rpc;
mod utils;

pub struct HostState {
    rpc: RwLock<HostRpc>,
}

pub struct PluginHost {
    data_folder: PathBuf,
    engine: Engine,
    linker: Linker<PluginState>,
    state: Arc<HostState>,
}

impl PluginHost {
    pub fn new(config: Config, data_folder: PathBuf) -> Result<Self, wasmtime::Error> {
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        configure_all(&mut linker);

        Ok(Self {
            data_folder,
            engine,
            linker,
            state: Arc::new(HostState {
                rpc: RwLock::new(HostRpc::new()),
            }),
        })
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

                if plugin_meta.name == "steel" {
                    error!("Skipping plugin 'steel': this name is reserved and cannot be loaded.",);
                    continue;
                }

                plugin_meta.file_path = file_path.canonicalize()?;
                plugins.push(plugin_meta);
            }
        }
        let (topology, invalid) = sort_plugins(plugins);
        if !invalid.is_empty() {
            error!("plugins with invalid dependencies: {:#?}", invalid);
        }

        Ok(topology)
    }

    pub async fn load_plugin(
        &mut self,
        plugin_meta: PluginMeta,
    ) -> Result<(Arc<PluginStore>, Instance), PluginLoaderError> {
        let bytes = read(&plugin_meta.file_path).await?;

        let precompiled = self.engine.precompile_module(&bytes)?;
        let module = unsafe { Module::deserialize(&self.engine, precompiled) }?;

        let plugin_data_folder = self.data_folder.join(&*plugin_meta.name);
        create_dir_all(&plugin_data_folder).await?;

        let wasi = WasiCtxBuilder::new()
            .preopened_dir(plugin_data_folder, "/", DirPerms::all(), FilePerms::all())?
            .build_p1();

        let state = PluginState::new(self.state.clone(), wasi, plugin_meta).await;
        let store = Arc::new(PluginStore {
            store: Mutex::new(Store::new(&self.engine, state)),
        });

        let instance = self.linker.instantiate(&mut *store.lock().await, &module)?;
        Ok((store, instance))
    }
}

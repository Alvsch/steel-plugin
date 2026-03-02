use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs::{create_dir_all, read, read_dir};
use tokio::sync::{Mutex, RwLock};
use tracing::error;
use wasmtime::{Engine, Linker, Memory, Module, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder, p1::WasiP1Ctx};

use crate::error::PluginLoaderError;
use crate::rpc::{HostRpc, PluginId};
use crate::utils::read_custom_section;
use crate::{
    EventRegistry, PluginExports, PluginMeta,
    instance::{PluginInstance, PluginStatus},
    utils,
};

pub struct PluginState {
    pub name: String,
    pub plugin_id: PluginId,
    pub wasi: WasiP1Ctx,
    pub registry: Arc<EventRegistry>,
    pub rpc: Arc<RwLock<HostRpc>>,
    pub exports: Option<PluginExports>,
    pub memory: Option<Memory>,
}

pub struct PluginLoader {
    engine: Engine,
    linker: Linker<PluginState>,
    data_dir: PathBuf,
    registry: Arc<EventRegistry>,
    rpc: Arc<RwLock<HostRpc>>,
}

impl PluginLoader {
    #[must_use]
    pub const fn new(
        engine: Engine,
        linker: Linker<PluginState>,
        data_dir: PathBuf,
        registry: Arc<EventRegistry>,
        rpc: Arc<RwLock<HostRpc>>,
    ) -> Self {
        Self {
            engine,
            linker,
            data_dir,
            registry,
            rpc,
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

                if plugin_meta.name == "steel" {
                    error!("Skipping plugin 'steel': this name is reserved and cannot be loaded.",);
                    continue;
                }

                plugin_meta.file_path = file_path.canonicalize()?;
                plugins.push(plugin_meta);
            }
        }
        let (topology, invalid) = utils::sort_plugins(plugins);
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

        let mut rpc = self.rpc.write().await;
        let plugin_id = rpc.next_id();
        let store = Store::try_new(
            &self.engine,
            PluginState {
                name: plugin_meta.name.clone(),
                plugin_id,
                wasi,
                registry: self.registry.clone(),
                rpc: self.rpc.clone(),
                exports: None,
                memory: None,
            },
        )?;
        let store = Arc::new(Mutex::new(store));
        rpc.register_plugin(plugin_id, plugin_meta.name.clone(), store.clone());

        let mut lock = store.lock().await;

        // init
        let instance = self.linker.instantiate_async(&mut *lock, &module).await?;
        let exports = PluginExports::resolve(&instance, &mut lock)
            .map_err(PluginLoaderError::PluginExportResolve)?;

        lock.data_mut().exports = Some(exports.clone());

        let memory = instance.get_memory(&mut *lock, "memory").unwrap();
        lock.data_mut().memory = Some(memory);

        drop(lock);

        Ok(PluginInstance {
            instance,
            meta: plugin_meta,
            status: PluginStatus::Disabled,
            exports,
            memory,
            store,
        })
    }
}

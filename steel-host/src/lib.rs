use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{create_dir_all, read};
use tokio::sync::RwLock;

use crate::error::PluginLoaderError;
use crate::linker::configure_all;
use crate::plugin::meta::PluginMeta;
use crate::plugin::{PluginState, PluginStore};
use crate::rpc::HostRpc;
use steel_plugin_sdk::utils::fat::FatPtr;
pub use wasmtime;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

pub mod error;
pub mod linker;
pub mod plugin;
pub mod rpc;
mod utils;

use crate::plugin::exports::PluginExports;
pub use utils::discover::discover_plugins;

pub const SCRATCH_SIZE: u32 = 4 * 1024;

pub struct HostState {
    rpc: RwLock<HostRpc>,
    enabled_plugins: RwLock<Vec<PluginStore>>,
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
                enabled_plugins: RwLock::new(Vec::new()),
            }),
        })
    }

    pub async fn load_plugin(
        &mut self,
        plugin_meta: PluginMeta,
    ) -> Result<PluginStore, PluginLoaderError> {
        let bytes = read(&plugin_meta.file_path).await?;

        let precompiled = self.engine.precompile_module(&bytes)?;
        let module = unsafe { Module::deserialize(&self.engine, precompiled) }?;

        let plugin_data_folder = self.data_folder.join(&*plugin_meta.name);
        create_dir_all(&plugin_data_folder).await?;

        let wasi = WasiCtxBuilder::new()
            .preopened_dir(plugin_data_folder, "/", DirPerms::all(), FilePerms::all())?
            .build_p1();

        let state = PluginState::new(self.state.clone(), wasi, plugin_meta).await;
        let mut store = Store::new(&self.engine, state);
        let instance = self.linker.instantiate_async(&mut store, &module).await?;
        let exports = PluginExports::resolve(instance, &mut store)?;

        // preallocate scratch
        let scratch_ptr = exports.alloc.call_async(&mut store, SCRATCH_SIZE).await?;

        let data = store.data_mut();
        data.scratch = FatPtr::new(scratch_ptr, SCRATCH_SIZE).unwrap();
        data.exports
            .set(Arc::new(exports))
            .map_err(|_| ())
            .expect("instance already initialized");

        Ok(PluginStore::new(store))
    }
}

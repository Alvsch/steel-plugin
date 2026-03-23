use std::path::PathBuf;
use std::sync::Arc;

use steel_plugin_sdk::utils::fat::FatPtr;
use tokio::fs::{create_dir_all, read};
use tokio::sync::Mutex;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

use crate::error::PluginLoaderError;
use crate::linker::configure_all;
use crate::plugin::{PluginExports, PluginMeta, PluginState, PluginStore};
use crate::state::HostState;

pub use utils::discover::discover_plugins;
pub use wasmtime;

pub mod error;
pub mod event;
pub mod linker;
pub mod plugin;
pub mod rpc;
mod state;
mod utils;

pub const SCRATCH_SIZE: u32 = 4 * 1024;

pub struct PluginHost {
    engine: Engine,
    linker: Linker<PluginState>,
    pub state: Arc<HostState>,
    data_folder: PathBuf,
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
            state: Arc::new(HostState::new()),
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

        let state = PluginState::new(self.state.clone(), wasi, plugin_meta);
        let mut store = Store::new(&self.engine, state);
        let instance = self.linker.instantiate_async(&mut store, &module).await?;
        let exports = PluginExports::resolve(instance, &mut store)?;

        // preallocate scratch
        let scratch_ptr = exports.alloc.call_async(&mut store, SCRATCH_SIZE).await?;

        let store = Arc::new(Mutex::new(store));
        {
            let mut lock = store.lock().await;
            let data = lock.data_mut();
            data.scratch = FatPtr::new(scratch_ptr, SCRATCH_SIZE).unwrap();
            data.exports
                .set(Arc::new(exports))
                .map_err(|_| ())
                .expect("exports already initialized");
            data.store
                .set(store.clone())
                .map_err(|_| ())
                .expect("store already initialized");
        }
        Ok(store)
    }
}

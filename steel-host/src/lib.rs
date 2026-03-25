use std::path::{Path, PathBuf};
use std::sync::Arc;

use steel_plugin_sdk::utils::fat::FatPtr;
use tokio::fs::{create_dir_all, read};
use tokio::sync::Mutex;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::p1::WasiP1Ctx;
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

use crate::error::{PluginLoaderError, PluginManagerError};
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

pub struct WasmEngine {
    engine: Engine,
    linker: Linker<PluginState>,
    data_folder: PathBuf,
}

impl WasmEngine {
    pub fn new(config: Config, data_folder: PathBuf) -> Result<Self, wasmtime::Error> {
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        configure_all(&mut linker);
        Ok(Self {
            engine,
            linker,
            data_folder,
        })
    }

    pub async fn preload_module(&self, file_path: &Path) -> Result<Module, wasmtime::Error> {
        let bytes = read(file_path).await?;
        let precompiled = self.engine.precompile_module(&bytes)?;
        let module = unsafe { Module::deserialize(&self.engine, precompiled) }?;
        Ok(module)
    }

    pub async fn prepare_wasi(&self, plugin_name: &str) -> Result<WasiP1Ctx, wasmtime::Error> {
        let plugin_data_folder = self.data_folder.join(plugin_name);
        create_dir_all(&plugin_data_folder).await?;

        let wasi = WasiCtxBuilder::new()
            .preopened_dir(&plugin_data_folder, "/", DirPerms::all(), FilePerms::all())?
            .build_p1();
        Ok(wasi)
    }

    async fn build_store(
        instance: Instance,
        mut store: Store<PluginState>,
    ) -> Result<PluginStore, wasmtime::Error> {
        let exports = PluginExports::resolve(instance, &mut store)?;

        // preallocate scratch
        let scratch_ptr = exports.alloc.call_async(&mut store, SCRATCH_SIZE).await?;

        let data = store.data_mut();
        data.scratch = FatPtr::new(scratch_ptr, SCRATCH_SIZE).unwrap();
        data.exports
            .set(Arc::new(exports))
            .map_err(|_| ())
            .expect("exports already initialized");

        let store = Arc::new(Mutex::new(store));
        {
            let mut lock = store.lock().await;
            let data = lock.data_mut();
            data.store
                .set(store.clone())
                .map_err(|_| ())
                .expect("store already initialized");
        }
        Ok(store)
    }

    pub async fn instantiate(
        &self,
        module: &Module,
        plugin_state: PluginState,
    ) -> Result<PluginStore, wasmtime::Error> {
        let mut store = Store::new(&self.engine, plugin_state);
        let instance = self.linker.instantiate_async(&mut store, module).await?;

        let store = Self::build_store(instance, store).await?;
        Ok(store)
    }
}

pub struct PluginHost {
    wasm: WasmEngine,
    pub state: Arc<HostState>,
}

impl PluginHost {
    pub fn new(config: Config, data_folder: PathBuf) -> Result<Self, wasmtime::Error> {
        Ok(Self {
            wasm: WasmEngine::new(config, data_folder)?,
            state: Arc::new(HostState::new()),
        })
    }

    pub async fn prepare_plugin(
        &self,
        plugin_meta: PluginMeta,
    ) -> Result<PluginStore, PluginLoaderError> {
        let module = self.wasm.preload_module(&plugin_meta.file_path).await?;
        let wasi = self.wasm.prepare_wasi(&plugin_meta.name).await?;

        let plugin_state = PluginState::new(self.state.clone(), wasi, plugin_meta);
        let plugin: PluginStore = self.wasm.instantiate(&module, plugin_state).await?;
        Ok(plugin)
    }

    pub async fn load_plugin(&self, plugin: &PluginStore) -> Result<(), PluginManagerError> {
        self.state.load_plugin(plugin).await
    }

    pub async fn enable_plugin(&self, plugin: &PluginStore) -> Result<(), PluginManagerError> {
        self.state.enable_plugin(plugin).await
    }
}

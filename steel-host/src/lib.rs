use std::fs::{create_dir_all, read};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use steel_plugin_core::PluginMeta;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::p2::add_to_linker_async;
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder};

use crate::error::{PluginContractError, PluginError};
use crate::linker::{HostLinker, PluginWorld};
use crate::plugin::{PluginState, PluginStore};
use crate::state::HostState;

pub use utils::discover::discover_plugins;
pub use wasmtime;

pub mod error;
pub mod interface;
pub mod linker;
pub mod plugin;
mod state;
mod utils;

#[expect(clippy::absolute_paths)]
pub type AsyncMutex<T> = tokio::sync::Mutex<T>;

pub struct Plugin {
    pub store: AsyncMutex<Store<PluginState>>,
    pub bindings: AsyncMutex<PluginWorld>,
}

pub struct WasmEngine {
    engine: Engine,
    linker: HostLinker,
    data_folder: PathBuf,
}

impl WasmEngine {
    pub fn new(config: Config, data_folder: PathBuf) -> wasmtime::Result<Self> {
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        add_to_linker_async(&mut linker)?;
        linker::add_to_linker(&mut linker)?;
        Ok(Self {
            engine,
            linker,
            data_folder,
        })
    }

    pub fn preload_component(&self, file_path: &Path) -> Result<Component, PluginError> {
        let bytes = read(file_path).map_err(|err| match err.kind() {
            ErrorKind::NotFound => PluginError::NotFound {
                file_path: file_path.to_path_buf(),
            },
            _ => PluginError::Io(err),
        })?;

        let precompiled = self
            .engine
            .precompile_component(&bytes)
            .map_err(PluginError::InvalidModule)?;

        let component = unsafe { Component::deserialize(&self.engine, precompiled) }
            .map_err(PluginError::InvalidModule)?;

        Ok(component)
    }

    pub fn prepare_wasi(&self, plugin_name: &str) -> Result<WasiCtx, PluginError> {
        let plugin_data_folder = self.data_folder.join(plugin_name);
        create_dir_all(&plugin_data_folder).map_err(PluginError::Io)?;

        let wasi = WasiCtxBuilder::new()
            .preopened_dir(&plugin_data_folder, "/", DirPerms::all(), FilePerms::all())
            .map_err(|err| {
                PluginError::Io(
                    err.downcast::<io::Error>()
                        .expect(".preopened_dir() can only return an io::Error"),
                )
            })?
            .build();

        Ok(wasi)
    }

    pub async fn instantiate(
        &self,
        component: &Component,
        plugin_state: PluginState,
    ) -> Result<PluginStore, PluginContractError> {
        let mut store = Store::new(&self.engine, plugin_state);

        let bindings = PluginWorld::instantiate_async(&mut store, component, &self.linker).await?;

        Ok(Arc::new(Plugin {
            store: AsyncMutex::new(store),
            bindings: AsyncMutex::new(bindings),
        }))
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
    ) -> Result<PluginStore, PluginError> {
        let module = self.wasm.preload_component(&plugin_meta.file_path)?;
        let wasi = self.wasm.prepare_wasi(&plugin_meta.name)?;

        let plugin_state = PluginState::new(self.state.clone(), wasi, plugin_meta);
        let plugin = self.wasm.instantiate(&module, plugin_state).await?;
        Ok(plugin)
    }

    pub async fn load_plugin(&self, plugin: &PluginStore) -> Result<(), PluginContractError> {
        self.state.load_plugin(plugin).await
    }

    pub async fn enable_plugin(&self, plugin: &PluginStore) -> Result<(), PluginContractError> {
        self.state.enable_plugin(plugin).await
    }

    pub async fn disable_plugin(&self, plugin: &PluginStore) -> Result<(), PluginContractError> {
        self.state.disable_plugin(plugin).await
    }
}

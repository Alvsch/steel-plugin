use std::path::{Path, PathBuf};
use std::sync::Arc;

use steel_plugin_core::PluginMeta;
use steel_utils::locks::AsyncMutex;
use wasmtime::Config;

use crate::engine::WasmEngine;
use crate::error::PluginError;
use crate::linker::PluginWorld;
use crate::plugin::{Plugin, PluginInstance, PluginState, PluginStore};
use crate::state::HostState;

pub use utils::discover::discover_plugins;
pub use wasmtime;

mod engine;
pub mod error;
pub mod linker;
pub mod plugin;
mod state;
mod utils;

pub struct PluginHost {
    engine: WasmEngine,
    pub state: Arc<HostState>,
}

impl PluginHost {
    pub fn new(config: Config, data_folder: PathBuf) -> Result<Self, wasmtime::Error> {
        Ok(Self {
            engine: WasmEngine::new(config, data_folder)?,
            state: Arc::new(HostState::new()),
        })
    }

    pub async fn load_plugin(
        &self,
        plugin_meta: PluginMeta,
        file_path: &Path,
    ) -> Result<PluginInstance, PluginError> {
        // compile
        let plugin = self.engine.compile_plugin(file_path).await?;

        // instantiate
        let wasi = self.engine.prepare_wasi(&plugin_meta.name).await?;
        let mut store = PluginStore::new(
            &self.engine.engine,
            PluginState::new(self.state.clone(), wasi, plugin_meta),
        );
        let bindings = PluginWorld::instantiate_async(&mut store, &plugin, &self.engine.linker)
            .await
            .map_err(PluginError::InvalidModule)?;

        let instance = Arc::new_cyclic(|weak| {
            store
                .data()
                .plugin
                .set(weak.clone())
                .expect("plugin already initialized");
            Plugin {
                store: AsyncMutex::new(store),
                bindings: AsyncMutex::new(bindings),
            }
        });

        self.state.register(&instance).await?;

        Ok(instance)
    }
}

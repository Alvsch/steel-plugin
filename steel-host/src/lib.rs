mod engine;
mod error;
mod linker;
mod plugin;
mod state;
mod utils;

use std::path::PathBuf;
use std::sync::Arc;

use crate::engine::WasmEngine;
use crate::linker::PluginWorld;
use crate::plugin::{PluginRuntime, PluginState};
use crate::state::HostState;

use steel_utils::locks::AsyncMutex;

pub use crate::error::{PluginContractError, PluginError};
pub use crate::plugin::{Plugin, PluginHandle, PluginStatus};
pub use crate::utils::discover::{DiscoveredPlugin, discover_plugins};
pub use steel_core::PluginApi;
pub use wasmtime::Config as WasmtimeConfig;

/// Owns the Wasmtime engine, loaded plugins, and host-side plugin services.
pub struct PluginHost {
    engine: WasmEngine,
    state: Arc<HostState>,
}

impl PluginHost {
    /// Create a host using Wasmtime's default configuration and the given data directory.
    pub fn new(data_folder: impl Into<PathBuf>) -> Result<Self, PluginError> {
        Self::with_config(WasmtimeConfig::new(), data_folder)
    }

    /// Create a host with a custom Wasmtime configuration.
    pub fn with_config(
        config: WasmtimeConfig,
        data_folder: impl Into<PathBuf>,
    ) -> Result<Self, PluginError> {
        Ok(Self {
            engine: WasmEngine::new(config, data_folder.into()).map_err(PluginError::Engine)?,
            state: Arc::new(HostState::new()),
        })
    }

    /// Compile and instantiate a discovered plugin.
    pub async fn load_plugin(
        &self,
        discovered: DiscoveredPlugin,
    ) -> Result<PluginHandle, PluginError> {
        let plugin = self.engine.compile_plugin(&discovered.path).await?;

        let wasi = self.engine.prepare_wasi(&discovered.meta.name).await?;
        let mut store = wasmtime::Store::new(
            &self.engine.engine,
            PluginState::new(self.state.clone(), wasi, discovered.meta),
        );
        let bindings = PluginWorld::instantiate_async(&mut store, &plugin, &self.engine.linker)
            .await
            .map_err(PluginError::InvalidModule)?;

        let name = store.data().meta.name.clone();
        let instance = Arc::new(Plugin {
            name,
            runtime: AsyncMutex::new(PluginRuntime { store, bindings }),
        });

        self.state.register(&instance).await?;

        Ok(instance)
    }

    /// Look up a loaded plugin by its declared name.
    #[must_use]
    pub fn plugin(&self, name: &str) -> Option<PluginHandle> {
        self.state.resolve_plugin(name)
    }

    /// Dispatch a Steel event to the plugins that registered for its topic.
    pub async fn dispatch_event(&self, event: PluginApi) -> Result<(), PluginContractError> {
        self.state
            .handler_registry
            .read()
            .await
            .dispatch_topic(event, &self.state)
            .await
            .map_err(PluginContractError::from)
    }
}

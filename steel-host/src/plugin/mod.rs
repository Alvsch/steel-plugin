mod resource;
mod state;

use steel_utils::locks::AsyncMutex;
use wasmtime::Store;

use crate::linker::PluginWorld;
use std::sync::Arc;

pub(crate) use crate::plugin::resource::PluginResources;
pub(crate) use crate::plugin::state::PluginState;
pub use crate::plugin::state::PluginStatus;

/// Shared handle to a loaded plugin.
pub type PluginHandle = Arc<Plugin>;
pub(crate) type PluginStore = Store<PluginState>;

pub struct Plugin {
    pub(crate) name: String,
    pub(crate) runtime: AsyncMutex<PluginRuntime>,
}

pub(crate) struct PluginRuntime {
    pub(crate) store: PluginStore,
    pub(crate) bindings: PluginWorld,
}

impl Plugin {
    /// The plugin's declared name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The plugin's current lifecycle status.
    pub async fn status(&self) -> PluginStatus {
        self.runtime.lock().await.store.data().status
    }

    /// Invoke the plugin's enable hook and mark it enabled.
    pub async fn enable(&self) -> Result<(), crate::error::PluginContractError> {
        let mut runtime = self.runtime.lock().await;
        let PluginRuntime { store, bindings } = &mut *runtime;
        bindings
            .host_plugin_sdk_plugin_api()
            .call_on_enable(&mut *store)
            .await?;

        store.data_mut().status = PluginStatus::Enabled;
        Ok(())
    }

    /// Invoke the plugin's disable hook and unregister it from the host.
    pub async fn disable(&self) -> Result<(), crate::error::PluginContractError> {
        let mut runtime = self.runtime.lock().await;
        let PluginRuntime { store, bindings } = &mut *runtime;
        bindings
            .host_plugin_sdk_plugin_api()
            .call_on_disable(&mut *store)
            .await?;

        let (host, name) = {
            let data = store.data_mut();
            data.status = PluginStatus::Disabled;
            (data.host.clone(), data.meta.name.clone())
        };
        drop(runtime);
        host.unregister_plugin(&name).await;
        Ok(())
    }
}

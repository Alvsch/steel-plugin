use std::sync::Arc;

use steel_utils::locks::AsyncMutex;
use wasmtime::Store;

use crate::linker::PluginWorld;
pub use crate::plugin::{
    resource::PluginResources,
    state::{PluginState, PluginStatus},
};

mod resource;
mod state;

pub type PluginInstance = Arc<Plugin>;
pub type PluginStore = Store<PluginState>;

pub struct Plugin {
    pub store: AsyncMutex<PluginStore>,
    pub bindings: AsyncMutex<PluginWorld>,
}

impl Plugin {
    pub async fn enable(&self) -> wasmtime::Result<()> {
        let mut store = self.store.lock().await;
        self.bindings
            .lock()
            .await
            .host_plugin_sdk_plugin_api()
            .call_on_enable(&mut *store)
            .await?;

        store.data_mut().status = PluginStatus::Enabled;
        Ok(())
    }

    pub async fn disable(&self) -> wasmtime::Result<()> {
        let mut store = self.store.lock().await;
        self.bindings
            .lock()
            .await
            .host_plugin_sdk_plugin_api()
            .call_on_disable(&mut *store)
            .await?;

        let data = store.data_mut();
        data.status = PluginStatus::Disabled;

        data.host.unregister_plugin(&data.meta.name);
        Ok(())
    }
}

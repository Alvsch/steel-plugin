use crate::error::PluginContractError;
use crate::linker::event::HandlerRegistry;
use crate::plugin::{PluginInstance, PluginStatus};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::warn;

pub struct HostState {
    pub handler_registry: RwLock<HandlerRegistry>,
    enabled_plugins: RwLock<Vec<PluginInstance>>,
    plugin_name: RwLock<HashMap<String, PluginInstance>>,
    next_id: AtomicU32,
}

impl Default for HostState {
    fn default() -> Self {
        Self::new()
    }
}

impl HostState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handler_registry: RwLock::new(HandlerRegistry::new()),
            enabled_plugins: RwLock::new(Vec::new()),
            plugin_name: RwLock::new(HashMap::new()),
            next_id: AtomicU32::new(1),
        }
    }

    pub fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    #[must_use]
    pub fn resolve_plugin(&self, plugin_name: &str) -> Option<PluginInstance> {
        self.plugin_name.read().get(plugin_name).cloned()
    }

    pub fn unregister_plugin(&self, plugin_name: &str) {
        let Some(_) = self.plugin_name.write().remove(plugin_name) else {
            warn!("attempted to unregister plugin '{plugin_name}' but it was not registered");
            return;
        };
    }

    pub async fn load_plugin(&self, plugin: &PluginInstance) -> Result<(), PluginContractError> {
        let store = plugin.store.lock().await;
        let data = store.data();

        // TODO: load information such as exposed rpc methods etc.
        self.plugin_name
            .write()
            .insert(data.meta.name.clone(), plugin.clone());

        Ok(())
    }

    pub async fn enable_plugin(&self, plugin: &PluginInstance) -> Result<(), PluginContractError> {
        let mut store = plugin.store.lock().await;
        plugin
            .bindings
            .lock()
            .await
            .host_plugin_sdk_plugin_api()
            .call_on_enable(&mut *store)
            .await?;

        store.data_mut().status = PluginStatus::Enabled;

        self.enabled_plugins.write().push(plugin.clone());
        Ok(())
    }

    pub async fn disable_plugin(&self, plugin: &PluginInstance) -> Result<(), PluginContractError> {
        let mut store = plugin.store.lock().await;

        plugin
            .bindings
            .lock()
            .await
            .host_plugin_sdk_plugin_api()
            .call_on_disable(&mut *store)
            .await?;

        store.data_mut().status = PluginStatus::Disabled;

        let mut enabled = self.enabled_plugins.write();
        enabled.retain(|p| !Arc::ptr_eq(p, plugin));

        self.unregister_plugin(&store.data().meta.name);
        Ok(())
    }
}

use crate::error::PluginContractError;
use crate::linker::event::HandlerRegistry;
use crate::plugin::PluginInstance;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use steel_utils::locks::{AsyncRwLock, SyncRwLock};
use tracing::warn;

pub struct HostState {
    pub handler_registry: AsyncRwLock<HandlerRegistry>,
    plugin_name: SyncRwLock<HashMap<String, PluginInstance>>,
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
            handler_registry: AsyncRwLock::new(HandlerRegistry::new()),
            plugin_name: SyncRwLock::new(HashMap::new()),
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

    pub async fn register(&self, plugin: &PluginInstance) -> Result<(), PluginContractError> {
        let store = plugin.store.lock().await;
        let data = store.data();

        // TODO: load information such as exposed rpc methods etc.
        self.plugin_name
            .write()
            .insert(data.meta.name.clone(), plugin.clone());

        Ok(())
    }
}

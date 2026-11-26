use crate::linker::event::HandlerRegistry;
use crate::plugin::PluginHandle;
use crate::{error::PluginContractError, plugin::Plugin};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
use steel_utils::locks::{AsyncRwLock, SyncRwLock};
use tracing::warn;

pub(crate) struct HostState {
    pub(crate) handler_registry: AsyncRwLock<HandlerRegistry>,
    plugins: SyncRwLock<HashMap<String, Weak<Plugin>>>,
}

impl Default for HostState {
    fn default() -> Self {
        Self::new()
    }
}

impl HostState {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            handler_registry: AsyncRwLock::new(HandlerRegistry::new()),
            plugins: SyncRwLock::new(HashMap::new()),
        }
    }

    #[must_use]
    pub(crate) fn resolve_plugin(&self, plugin_name: &str) -> Option<PluginHandle> {
        self.plugins.read().get(plugin_name).and_then(Weak::upgrade)
    }

    pub(crate) async fn unregister_plugin(&self, plugin_name: &str) {
        if self.plugins.write().remove(plugin_name).is_none() {
            warn!("attempted to unregister plugin '{plugin_name}' but it was not registered");
            return;
        }

        self.handler_registry
            .write()
            .await
            .unsubscribe_plugin(plugin_name);
    }

    pub(crate) async fn register(&self, plugin: &PluginHandle) -> Result<(), PluginContractError> {
        let runtime = plugin.runtime.lock().await;
        let store = &runtime.store;
        let data = store.data();

        self.plugins
            .write()
            .insert(data.meta.name.clone(), Arc::downgrade(plugin));

        Ok(())
    }
}

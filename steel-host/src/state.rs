use crate::event::handler::HandlerRegistry;
use crate::plugin::{PluginState, PluginStore};
use crate::rpc::{HostRpc, PluginRpc};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use steel_plugin_sdk::rpc::PluginId;
use tokio::sync::{Mutex, RwLock};
use wasmtime::Store;

pub struct HostState {
    pub rpc: RwLock<HostRpc>,
    pub handler_registry: RwLock<HandlerRegistry>,
    pub enabled_plugins: RwLock<Vec<PluginStore>>,
    plugin_name: RwLock<HashMap<String, PluginId>>,
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
            rpc: RwLock::new(HostRpc::new()),
            handler_registry: RwLock::new(HandlerRegistry::new()),
            enabled_plugins: RwLock::new(Vec::new()),
            plugin_name: RwLock::new(HashMap::new()),
            next_id: AtomicU32::new(0),
        }
    }

    pub fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    #[must_use]
    pub async fn resolve_plugin(&self, plugin_name: &str) -> Option<PluginId> {
        self.plugin_name.read().await.get(plugin_name).copied()
    }

    pub async fn register_plugin(
        &self,
        plugin_id: PluginId,
        plugin_name: String,
        store: Arc<Mutex<Store<PluginState>>>,
    ) {
        self.rpc
            .write()
            .await
            .plugins
            .insert(plugin_id, PluginRpc::new(store.clone()));
        self.plugin_name
            .write()
            .await
            .insert(plugin_name, plugin_id);
    }

    pub async fn unregister_plugin(&self, plugin_name: &str) {
        let plugin_id = self.plugin_name.write().await.remove(plugin_name).unwrap();
        self.rpc.write().await.plugins.remove(&plugin_id);
    }
}

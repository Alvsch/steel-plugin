use crate::error::PluginContractError;
use crate::interface::event::HandlerRegistry;
use crate::interface::objects::{ObjectHandler, ObjectRegistry};
use crate::interface::rpc::{HostRpc, PluginRpc};
use crate::plugin::{PluginStatus, PluginStore};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use steel_plugin_sdk::export::{ExportedId, ExportedKind};
use steel_plugin_sdk::objects::HandleKey;
use steel_plugin_sdk::rpc::PluginId;
use tracing::warn;

pub struct HostState {
    pub objects: RwLock<ObjectRegistry>,
    pub rpc: RwLock<HostRpc>,
    pub handler_registry: RwLock<HandlerRegistry>,
    enabled_plugins: RwLock<Vec<PluginStore>>,
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
            objects: RwLock::new(ObjectRegistry::new()),
            rpc: RwLock::new(HostRpc::new()),
            handler_registry: RwLock::new(HandlerRegistry::new()),
            enabled_plugins: RwLock::new(Vec::new()),
            plugin_name: RwLock::new(HashMap::new()),
            next_id: AtomicU32::new(1),
        }
    }

    pub fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn register_object_handler(&self, handler: ObjectHandler) -> HandleKey {
        self.objects.write().register(handler)
    }

    pub fn unregister_object_handler(&self, key: HandleKey) -> Option<ObjectHandler> {
        self.objects.write().unregister(key)
    }

    #[must_use]
    pub fn resolve_plugin(&self, plugin_name: &str) -> Option<PluginId> {
        self.plugin_name.read().get(plugin_name).copied()
    }

    pub fn unregister_plugin(&self, plugin_name: &str) {
        let Some(plugin_id) = self.plugin_name.write().remove(plugin_name) else {
            warn!("attempted to unregister plugin '{plugin_name}' but it was not registered");
            return;
        };
        self.rpc.write().plugins.remove(&plugin_id);
    }

    pub async fn load_plugin(&self, plugin: &PluginStore) -> Result<(), PluginContractError> {
        let mut store = plugin.store.lock().await;
        let data = store.data();

        // register plugin
        self.rpc
            .write()
            .plugins
            .insert(data.plugin_id, PluginRpc::new(plugin.clone()));
        self.plugin_name
            .write()
            .insert(data.meta.name.clone(), data.plugin_id);

        // gather exported functions
        let exported_ids: Vec<ExportedId> = {
            let data = plugin
                .bindings
                .lock()
                .await
                .host_plugin_sdk_plugin_api()
                .call_on_load(&mut *store)
                .await?;

            rmp_serde::from_slice(&data)
                .map_err(|_| PluginContractError::Other("invalid load data".to_string()))?
        };

        // resolve and register exported functions
        for exported in exported_ids {
            match exported.kind {
                ExportedKind::Rpc { export_name } => {
                    let data = store.data();
                    let plugin_id = data.plugin_id;
                    let method_id = data.host.next_id();
                    data.host
                        .rpc
                        .write()
                        .get_plugin_mut(plugin_id)
                        .expect("plugin should be registered")
                        .register_method(method_id, export_name.to_string(), exported.id);
                }
                ExportedKind::Event { topic_id, priority } => {
                    self.handler_registry.write().subscribe(
                        topic_id,
                        plugin.clone(),
                        exported.id,
                        priority,
                    );
                }
                ExportedKind::Command => todo!(),
            }
        }

        Ok(())
    }

    pub async fn enable_plugin(&self, plugin: &PluginStore) -> Result<(), PluginContractError> {
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

    pub async fn disable_plugin(&self, plugin: &PluginStore) -> Result<(), PluginContractError> {
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

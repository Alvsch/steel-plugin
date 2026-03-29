use crate::error::{PluginContractError, PluginError};
use crate::event::handler::{HandlerFn, HandlerRegistry};
use crate::plugin::{PluginStatus, PluginStore};
use crate::rpc::{HostRpc, PluginRpc};
use crate::utils::memory::PluginMemory;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use steel_plugin_sdk::export::{ExportedId, ExportedKind};
use steel_plugin_sdk::rpc::PluginId;
use tokio::sync::RwLock;
use tracing::warn;

pub struct HostState {
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
            rpc: RwLock::new(HostRpc::new()),
            handler_registry: RwLock::new(HandlerRegistry::new()),
            enabled_plugins: RwLock::new(Vec::new()),
            plugin_name: RwLock::new(HashMap::new()),
            next_id: AtomicU32::new(1),
        }
    }

    pub fn next_id(&self) -> NonZeroU32 {
        let next_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        NonZeroU32::new(next_id).expect("next_id cant be zero")
    }

    #[must_use]
    pub async fn resolve_plugin(&self, plugin_name: &str) -> Option<PluginId> {
        self.plugin_name.read().await.get(plugin_name).copied()
    }

    pub async fn unregister_plugin(&self, plugin_name: &str) {
        let Some(plugin_id) = self.plugin_name.write().await.remove(plugin_name) else {
            warn!("attempted to unregister plugin '{plugin_name}' but it was not registered");
            return;
        };
        self.rpc.write().await.plugins.remove(&plugin_id);
    }

    pub async fn load_plugin(&self, plugin: &PluginStore) -> Result<(), PluginContractError> {
        let mut store = plugin.lock().await;
        let data = store.data();
        let exports = data.exports().clone();

        // register plugin
        self.rpc
            .write()
            .await
            .plugins
            .insert(data.plugin_id, PluginRpc::new(plugin.clone()));
        self.plugin_name
            .write()
            .await
            .insert(data.meta.name.clone(), data.plugin_id);

        // gather exported functions
        let exported_ids: Vec<ExportedId> = {
            let data_ptr = exports.on_load(&mut store).await?;
            let memory = PluginMemory::new(&mut *store, &exports.memory);
            let data = memory.read(data_ptr);

            rmp_serde::from_slice(data)
                .map_err(|_| PluginContractError::Other("invalid load data".to_string()))?
        };

        let table = exports
            .instance
            .get_table(&mut *store, "__indirect_function_table")
            .ok_or(PluginContractError::Other(
                "missing '__indirect_function_table'".to_string(),
            ))?;

        // resolve and register exported functions
        for exported in exported_ids {
            let func_ref = table
                .get(&mut *store, u64::from(exported.id))
                .ok_or_else(|| PluginContractError::Other("invalid export id".to_string()))?;

            let func = func_ref
                .as_func()
                .ok_or_else(|| PluginContractError::Other("export not a function".to_string()))?
                .ok_or_else(|| PluginContractError::Other("null function export".to_string()))?;

            let typed_func: HandlerFn = func.typed(&mut *store)?;

            match exported.kind {
                ExportedKind::Rpc { export_name } => {
                    let data = store.data();
                    let plugin_id = data.plugin_id;
                    let method_id = data.host.next_id();
                    data.host
                        .rpc
                        .write()
                        .await
                        .get_plugin_mut(plugin_id)
                        .expect("plugin should be registered")
                        .register_method(method_id, export_name.to_string(), typed_func);
                }
                ExportedKind::Event { topic_id, priority } => {
                    self.handler_registry.write().await.subscribe(
                        topic_id,
                        plugin.clone(),
                        typed_func,
                        priority,
                    );
                }
                ExportedKind::Command => todo!(),
            }
        }

        Ok(())
    }

    pub async fn enable_plugin(&self, plugin: &PluginStore) -> Result<(), PluginError> {
        let store = &mut *plugin.lock().await;
        let exports = store.data().exports().clone();

        exports.on_enable(&mut *store).await?;
        store.data_mut().status = PluginStatus::Enabled;

        let host = &store.data().host;
        host.enabled_plugins.write().await.push(plugin.clone());

        Ok(())
    }

    pub async fn disable_plugin(&self, plugin: &PluginStore) -> Result<(), PluginError> {
        let store = &mut *plugin.lock().await;
        let exports = store.data().exports().clone();

        exports.on_disable(&mut *store).await?;
        store.data_mut().status = PluginStatus::Disabled;

        let data = store.data();
        let mut enabled = data.host.enabled_plugins.write().await;
        enabled.retain(|p| !Arc::ptr_eq(p, plugin));

        data.host.unregister_plugin(&data.meta.name).await;
        Ok(())
    }
}

use crate::error::PluginManagerError;
use crate::plugin::exports::PluginExports;
use crate::{HostState, PluginMeta};
use std::cell::OnceCell;
use std::sync::Arc;
use steel_plugin_sdk::rpc::PluginId;
use steel_plugin_sdk::utils::fat::FatPtr;
use tokio::sync::{Mutex, MutexGuard};
use wasmtime::Store;
use wasmtime_wasi::p1::WasiP1Ctx;

pub mod exports;
pub mod meta;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    Enabled,
    Disabled,
}

pub struct PluginState {
    pub host: Arc<HostState>,
    pub plugin_id: PluginId,
    pub meta: PluginMeta,
    pub status: PluginStatus,
    pub wasi: WasiP1Ctx,
    pub exports: OnceCell<Arc<PluginExports>>,
    pub scratch: FatPtr,
}

impl PluginState {
    pub async fn new(host: Arc<HostState>, wasi: WasiP1Ctx, meta: PluginMeta) -> Self {
        let plugin_id = host.rpc.write().await.next_id();
        Self {
            host,
            plugin_id,
            meta,
            status: PluginStatus::Disabled,
            wasi,
            exports: OnceCell::new(),
            scratch: FatPtr::new(1, 1).unwrap(),
        }
    }

    pub fn exports(&self) -> &Arc<PluginExports> {
        self.exports.get().expect("instance not yet initialized")
    }
}

#[derive(Clone)]
pub struct PluginStore {
    inner: Arc<Mutex<Store<PluginState>>>,
}

impl PluginStore {
    #[must_use]
    pub fn new(store: Store<PluginState>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(store)),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, Store<PluginState>> {
        self.inner.lock().await
    }

    pub async fn enable_plugin(&self) -> Result<(), PluginManagerError> {
        let store = &mut *self.lock().await;
        let exports = store.data().exports().clone();

        let data = store.data();
        data.host.rpc.write().await.register_plugin(
            data.plugin_id,
            data.meta.name.clone(),
            self.inner.clone(),
        );

        exports.on_enable.call_async(&mut *store, ()).await?;

        store.data_mut().status = PluginStatus::Enabled;

        let host = &store.data().host;
        host.enabled_plugins.write().await.push(self.clone());

        Ok(())
    }

    pub async fn disable_plugin(&self) -> Result<(), PluginManagerError> {
        let store = &mut *self.lock().await;
        let exports = store.data().exports().clone();

        exports.on_disable.call_async(&mut *store, ()).await?;
        store.data_mut().status = PluginStatus::Disabled;

        let data = store.data();
        let mut enabled = data.host.enabled_plugins.write().await;
        enabled.retain(|p| !Arc::ptr_eq(&p.inner, &self.inner));

        let mut rpc = data.host.rpc.write().await;
        rpc.unregister_plugin(&data.meta.name);
        Ok(())
    }
}

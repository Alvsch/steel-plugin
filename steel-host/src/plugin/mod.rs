use crate::error::PluginManagerError;
use crate::rpc::PluginId;
use crate::{HostState, PluginMeta};
use std::cell::OnceCell;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use wasmtime::{Instance, Store};
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
    pub instance: OnceCell<Instance>,
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
            instance: OnceCell::new(),
        }
    }

    pub fn instance(&self) -> Instance {
        *self.instance.get().expect("instance not yet initialized")
    }
}

#[derive(Clone)]
pub struct PluginStore {
    inner: Arc<Mutex<(Store<PluginState>, Instance)>>,
}

impl PluginStore {
    #[must_use]
    pub fn new(store: Store<PluginState>, instance: Instance) -> Self {
        Self {
            inner: Arc::new(Mutex::new((store, instance))),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, (Store<PluginState>, Instance)> {
        self.inner.lock().await
    }

    pub async fn enable_plugin(&self) -> Result<(), PluginManagerError> {
        let (store, instance) = &mut *self.lock().await;
        let on_enable = instance.get_typed_func::<(), ()>(&mut *store, "on_enable")?;
        on_enable.call_async(&mut *store, ()).await?;

        store.data_mut().status = PluginStatus::Enabled;

        let host = &store.data().host;
        host.enabled_plugins.write().await.push(self.clone());

        Ok(())
    }
}

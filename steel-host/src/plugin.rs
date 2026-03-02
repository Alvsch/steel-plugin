use crate::instance::PluginStatus;
use crate::rpc::PluginId;
use crate::{HostState, PluginMeta};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use wasmtime::{Engine, Store};
use wasmtime_wasi::p1::WasiP1Ctx;

pub struct PluginState {
    pub host: Arc<HostState>,
    pub plugin_id: PluginId,
    pub meta: PluginMeta,
    pub status: PluginStatus,
    pub wasi: WasiP1Ctx,
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
        }
    }
}

pub struct PluginStore {
    store: Mutex<Store<PluginState>>,
}

impl PluginStore {
    pub fn new(engine: &Engine, state: PluginState) -> Self {
        Self {
            store: Mutex::new(Store::new(engine, state)),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, Store<PluginState>> {
        self.store.lock().await
    }
}

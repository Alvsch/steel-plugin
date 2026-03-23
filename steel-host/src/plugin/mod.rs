use crate::state::HostState;
use std::cell::OnceCell;
use std::sync::Arc;
use steel_plugin_sdk::rpc::PluginId;
use steel_plugin_sdk::utils::fat::FatPtr;
use tokio::sync::Mutex;
use wasmtime::Store;
use wasmtime_wasi::p1::WasiP1Ctx;

pub use exports::{AllocFunc, DeallocFunc, PluginExports};
pub use meta::PluginMeta;

mod exports;
mod meta;

pub type PluginStore = Arc<Mutex<Store<PluginState>>>;

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
    pub store: OnceCell<PluginStore>,
}

impl PluginState {
    pub fn new(host: Arc<HostState>, wasi: WasiP1Ctx, meta: PluginMeta) -> Self {
        let plugin_id = host.next_id();
        Self {
            host,
            plugin_id,
            meta,
            status: PluginStatus::Disabled,
            wasi,
            exports: OnceCell::new(),
            scratch: FatPtr::new(1, 1).unwrap(),
            store: OnceCell::new(),
        }
    }

    pub fn exports(&self) -> &Arc<PluginExports> {
        self.exports.get().expect("exports not yet initialized")
    }

    pub fn store(&self) -> &PluginStore {
        self.store.get().expect("store not yet initialized")
    }
}

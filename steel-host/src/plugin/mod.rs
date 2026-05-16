use crate::{Plugin, state::HostState};
use parking_lot::Mutex;
use std::{cell::OnceCell, sync::Arc};
use steel_plugin_core::PluginMeta;
use steel_plugin_sdk::rpc::PluginId;
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

pub type PluginStore = Arc<Plugin>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    Enabled,
    Disabled,
}

pub struct PluginState {
    pub wasi: WasiCtx,
    pub table: ResourceTable,
    pub host: Arc<HostState>,
    pub plugin_id: PluginId,
    pub meta: PluginMeta,
    pub status: PluginStatus,
    pub plugin: OnceCell<Arc<Mutex<Plugin>>>,
}

impl PluginState {
    pub fn new(host: Arc<HostState>, wasi: WasiCtx, meta: PluginMeta) -> Self {
        let plugin_id = PluginId(host.next_id());
        Self {
            wasi,
            table: ResourceTable::new(),
            host,
            plugin_id,
            meta,
            status: PluginStatus::Disabled,
            plugin: OnceCell::new(),
        }
    }

    pub fn plugin(&self) -> Arc<Mutex<Plugin>> {
        self.plugin.get().expect("plugin not initialized").clone()
    }
}

impl WasiView for PluginState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

use std::{cell::OnceCell, sync::Arc};

use steel_plugin_core::PluginMeta;
use steel_plugin_sdk::rpc::PluginId;
use steel_utils::locks::SyncMutex;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::{
    plugin::{Plugin, resource::PluginResources},
    state::HostState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    Enabled,
    Disabled,
}

pub struct PluginState {
    pub wasi: WasiCtx,
    pub resources: PluginResources,
    pub host: Arc<HostState>,
    pub plugin_id: PluginId,
    pub meta: PluginMeta,
    pub status: PluginStatus,
    pub plugin: OnceCell<Arc<SyncMutex<Plugin>>>,
}

impl PluginState {
    pub fn new(host: Arc<HostState>, wasi: WasiCtx, meta: PluginMeta) -> Self {
        let plugin_id = PluginId(host.next_id());
        Self {
            wasi,
            resources: PluginResources::new(),
            host,
            plugin_id,
            meta,
            status: PluginStatus::Disabled,
            plugin: OnceCell::new(),
        }
    }

    pub fn plugin(&self) -> Arc<SyncMutex<Plugin>> {
        self.plugin.get().expect("plugin not initialized").clone()
    }
}

impl WasiView for PluginState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.resources.table,
        }
    }
}

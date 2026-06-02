use std::{
    cell::OnceCell,
    sync::{Arc, Weak},
};

use steel_plugin_core::PluginMeta;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::{
    plugin::{Plugin, PluginInstance, resource::PluginResources},
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
    pub meta: PluginMeta,
    pub status: PluginStatus,
    pub plugin: OnceCell<Weak<Plugin>>,
}

impl PluginState {
    pub fn new(host: Arc<HostState>, wasi: WasiCtx, meta: PluginMeta) -> Self {
        Self {
            wasi,
            resources: PluginResources::new(),
            host,
            meta,
            status: PluginStatus::Disabled,
            plugin: OnceCell::new(),
        }
    }

    pub fn plugin(&self) -> PluginInstance {
        self.plugin
            .get()
            .expect("plugin not initialized")
            .upgrade()
            .expect("plugin no longer initialized")
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

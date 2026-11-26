use std::sync::Arc;

use steel_plugin_core::PluginMeta;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::{plugin::resource::PluginResources, state::HostState};

/// Lifecycle state of a loaded plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    /// The plugin's enable hook has completed.
    Enabled,
    /// The plugin is loaded but not enabled, or has been disabled.
    Disabled,
}

pub(crate) struct PluginState {
    pub(crate) wasi: WasiCtx,
    pub(crate) resources: PluginResources,
    pub(crate) host: Arc<HostState>,
    pub(crate) meta: PluginMeta,
    pub(crate) status: PluginStatus,
}

impl PluginState {
    pub(crate) fn new(host: Arc<HostState>, wasi: WasiCtx, meta: PluginMeta) -> Self {
        Self {
            wasi,
            resources: PluginResources::new(),
            host,
            meta,
            status: PluginStatus::Disabled,
        }
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

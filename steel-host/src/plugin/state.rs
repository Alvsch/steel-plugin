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

    pub(crate) fn declares_dependency(&self, plugin_name: &str) -> bool {
        self.meta
            .depends
            .iter()
            .any(|dependency| dependency == plugin_name)
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use semver::Version;
    use steel_plugin_core::{PluginMeta, STEEL_API_VERSION};
    use wasmtime_wasi::WasiCtxBuilder;

    use crate::state::HostState;

    use super::PluginState;

    #[test]
    fn rpc_access_requires_a_direct_dependency() {
        let state = PluginState::new(
            Arc::new(HostState::new()),
            WasiCtxBuilder::new().build(),
            PluginMeta {
                api_version: STEEL_API_VERSION.clone(),
                name: "consumer".to_owned(),
                description: String::new(),
                version: Version::new(0, 1, 0),
                authors: Vec::new(),
                depends: vec!["provider".to_owned()],
            },
        );

        assert!(state.declares_dependency("provider"));
        assert!(!state.declares_dependency("transitive-provider"));
        assert!(!state.declares_dependency("unlisted"));
    }
}

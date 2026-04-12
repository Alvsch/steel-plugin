use std::collections::BTreeMap;
use steel_plugin_sdk::rpc::{MethodId, PluginId};

use crate::interface::rpc::PluginRpc;

pub struct HostRpc {
    pub plugins: BTreeMap<PluginId, PluginRpc>,
}

impl Default for HostRpc {
    fn default() -> Self {
        Self::new()
    }
}

impl HostRpc {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            plugins: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn resolve_method(&self, plugin_id: PluginId, method_name: &str) -> Option<MethodId> {
        self.plugins
            .get(&plugin_id)
            .and_then(|plugin| plugin.method_name.get(method_name))
            .copied()
    }

    #[must_use]
    pub fn get_plugin(&self, plugin_id: PluginId) -> Option<&PluginRpc> {
        self.plugins.get(&plugin_id)
    }

    #[must_use]
    pub fn get_plugin_mut(&mut self, plugin_id: PluginId) -> Option<&mut PluginRpc> {
        self.plugins.get_mut(&plugin_id)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use semver::Version;
    use steel_plugin_core::PluginMeta;
    use steel_plugin_sdk::rpc::{MethodId, PluginId};
    use tokio::sync::Mutex;
    use wasmtime::{Engine, Func, Store};
    use wasmtime_wasi::WasiCtxBuilder;

    use super::HostRpc;
    use crate::interface::rpc::PluginRpc;
    use crate::plugin::PluginState;
    use crate::state::HostState;

    fn make_plugin_rpc_with_method(method_name: &str, method_id: MethodId) -> PluginRpc {
        let host = Arc::new(HostState::new());
        let wasi = WasiCtxBuilder::new().build_p1();
        let meta = PluginMeta {
            name: "plugin-under-test".to_string(),
            description: String::new(),
            version: Version::new(0, 1, 0),
            authors: Vec::new(),
            depends: Vec::new(),
            api_version: Version::new(0, 2, 0),
            file_path: PathBuf::new(),
        };

        let state = PluginState::new(host, wasi, meta);
        let engine = Engine::default();
        let mut store = Store::new(&engine, state);

        let func = Func::wrap(&mut store, |value: u64| -> u64 { value });
        let typed = func
            .typed::<u64, u64>(&mut store)
            .expect("function should have expected signature");

        let mut plugin_rpc = PluginRpc::new(Arc::new(Mutex::new(store)));
        plugin_rpc.register_method(method_id, method_name.to_string(), typed);
        plugin_rpc
    }

    #[test]
    fn resolve_method_returns_registered_method_id() {
        let plugin_id = PluginId::new(1).expect("plugin id should be non-zero");
        let method_id = MethodId::new(7).expect("method id should be non-zero");

        let mut host_rpc = HostRpc::new();
        host_rpc
            .plugins
            .insert(plugin_id, make_plugin_rpc_with_method("echo", method_id));

        assert_eq!(host_rpc.resolve_method(plugin_id, "echo"), Some(method_id));
        assert_eq!(host_rpc.resolve_method(plugin_id, "missing"), None);
    }

    #[test]
    fn get_plugin_and_get_plugin_mut_observe_registry_state() {
        let plugin_id = PluginId::new(1).expect("plugin id should be non-zero");
        let method_id = MethodId::new(1).expect("method id should be non-zero");

        let mut host_rpc = HostRpc::new();
        assert!(host_rpc.get_plugin(plugin_id).is_none());
        assert!(host_rpc.get_plugin_mut(plugin_id).is_none());

        host_rpc
            .plugins
            .insert(plugin_id, make_plugin_rpc_with_method("echo", method_id));

        assert!(host_rpc.get_plugin(plugin_id).is_some());
        assert!(host_rpc.get_plugin_mut(plugin_id).is_some());
    }
}

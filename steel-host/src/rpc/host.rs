use crate::rpc::PluginRpc;
use std::collections::BTreeMap;
use steel_plugin_sdk::rpc::{MethodId, PluginId};

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

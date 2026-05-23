use std::collections::BTreeMap;
use steel_plugin_sdk::rpc::PluginId;

use crate::plugin::PluginInstance;

pub struct RpcRegistry {
    pub plugins: BTreeMap<PluginId, PluginInstance>,
}

impl Default for RpcRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            plugins: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn get_plugin(&self, plugin_id: PluginId) -> Option<&PluginInstance> {
        self.plugins.get(&plugin_id)
    }
}

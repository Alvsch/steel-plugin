use std::collections::BTreeMap;

use crate::{linker::host::plugin_sdk::rpc::PluginId, plugin::PluginInstance};

pub struct RpcRegistry {
    pub plugins: BTreeMap<u32, PluginInstance>,
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
        self.plugins.get(&plugin_id.id)
    }
}

use crate::PluginState;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use steel_plugin_sdk::rpc::{MethodId, PluginId};
use tokio::sync::Mutex;
use wasmtime::{Store, TypedFunc};

pub type RpcMethod = TypedFunc<u64, u64>;

pub struct PluginRpc {
    pub store: Arc<Mutex<Store<PluginState>>>,
    methods: BTreeMap<MethodId, RpcMethod>,
    method_name: HashMap<String, MethodId>,
}

impl PluginRpc {
    fn new(store: Arc<Mutex<Store<PluginState>>>) -> Self {
        Self {
            store,
            methods: BTreeMap::new(),
            method_name: HashMap::new(),
        }
    }

    pub fn register_method(&mut self, method_id: MethodId, method_name: String, method: RpcMethod) {
        self.methods.insert(method_id, method);
        self.method_name.insert(method_name, method_id);
    }

    #[must_use]
    pub fn get_method(&self, method_id: MethodId) -> &RpcMethod {
        self.methods.get(&method_id).unwrap()
    }
}

pub struct HostRpc {
    plugins: BTreeMap<PluginId, PluginRpc>,
    plugin_name: HashMap<String, PluginId>,
}

impl Default for HostRpc {
    fn default() -> Self {
        Self::new()
    }
}

impl HostRpc {
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: BTreeMap::new(),
            plugin_name: HashMap::new(),
        }
    }

    #[must_use]
    pub fn resolve_plugin(&self, plugin_name: &str) -> Option<PluginId> {
        self.plugin_name.get(plugin_name).copied()
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

    pub fn register_plugin(
        &mut self,
        plugin_id: PluginId,
        plugin_name: String,
        store: Arc<Mutex<Store<PluginState>>>,
    ) {
        self.plugins.insert(plugin_id, PluginRpc::new(store));
        self.plugin_name.insert(plugin_name, plugin_id);
    }

    pub fn unregister_plugin(&mut self, plugin_name: &str) {
        let plugin_id = self.plugin_name.remove(plugin_name).unwrap();
        self.plugins.remove(&plugin_id);
    }
}

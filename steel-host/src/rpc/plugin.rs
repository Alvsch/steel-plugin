use crate::plugin::PluginStore;
use crate::rpc::RpcMethod;
use std::collections::{BTreeMap, HashMap};
use steel_plugin_sdk::rpc::MethodId;

pub struct PluginRpc {
    pub store: PluginStore,
    pub methods: BTreeMap<MethodId, RpcMethod>,
    pub method_name: HashMap<String, MethodId>,
}

impl PluginRpc {
    pub(crate) fn new(store: PluginStore) -> Self {
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

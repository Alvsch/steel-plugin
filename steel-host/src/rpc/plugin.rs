use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use steel_plugin_sdk::rpc::MethodId;
use tokio::sync::Mutex;
use wasmtime::Store;

use crate::{plugin::PluginState, rpc::RpcMethod};

#[derive(Clone)]
pub struct PluginRpc {
    pub store: Arc<Mutex<Store<PluginState>>>,
    pub methods: BTreeMap<MethodId, RpcMethod>,
    pub method_name: HashMap<String, MethodId>,
}

impl PluginRpc {
    pub fn new(store: Arc<Mutex<Store<PluginState>>>) -> Self {
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

use crate::sdk;

#[doc(hidden)]
pub mod export {
    pub use inventory::{iter, submit};

    pub struct RpcMethod {
        pub name: &'static str,
        pub function: fn(&[u8]) -> Option<Vec<u8>>,
    }

    inventory::collect!(RpcMethod);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PluginId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodId(pub u32);

#[allow(clippy::must_use_candidate)]
pub fn dispatch(plugin_id: PluginId, method_name: &str, data: &[u8]) -> Option<Vec<u8>> {
    sdk::rpc::dispatch(plugin_id.0, method_name, data)
}

pub fn resolve_plugin(plugin_name: &str) -> Option<PluginId> {
    sdk::rpc::resolve_plugin(plugin_name).map(PluginId)
}

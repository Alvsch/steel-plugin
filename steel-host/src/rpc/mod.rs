pub type RpcMethod = wasmtime::TypedFunc<u64, u64>;

pub use host::HostRpc;
pub use plugin::PluginRpc;

mod host;
mod plugin;

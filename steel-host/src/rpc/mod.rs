pub mod host;
pub mod plugin;

pub type RpcMethod = wasmtime::TypedFunc<u64, u64>;

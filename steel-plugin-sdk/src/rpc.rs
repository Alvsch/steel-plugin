pub use crate::sdk::rpc::{dispatch, resolve_plugin};

#[doc(hidden)]
pub mod export {
    pub use inventory::{iter, submit};

    pub struct RpcMethod {
        pub name: &'static str,
        pub function: fn(&[u8]) -> Option<Vec<u8>>,
    }

    inventory::collect!(RpcMethod);
}

use steel_plugin_core::TopicId;

use crate::event::Event;

#[doc(hidden)]
pub struct EventHandler {
    pub topic_id: TopicId,
    pub priority: i8,
    pub function: fn(&mut Event),
}

inventory::collect!(EventHandler);

pub type RpcFunction = fn(&[u8]) -> Result<Option<Vec<u8>>, crate::sdk::rpc::RpcError>;

#[doc(hidden)]
pub struct RpcMethod {
    pub name: &'static str,
    pub function: RpcFunction,
}

inventory::collect!(RpcMethod);

use steel_plugin_core::TopicId;

use crate::event::Event;

#[doc(hidden)]
pub struct EventHandler {
    pub id: u32,
    pub topic_id: TopicId,
    pub priority: i8,
    pub function: fn(&mut Event),
}

inventory::collect!(EventHandler);

#[doc(hidden)]
pub struct RpcMethod {
    pub name: &'static str,
    pub function: fn(&[u8]) -> Option<Vec<u8>>,
}

inventory::collect!(RpcMethod);

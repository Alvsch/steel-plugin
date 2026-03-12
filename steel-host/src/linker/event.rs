use crate::plugin::PluginState;
use steel_plugin_sdk::event::TopicId;
use wasmtime::Caller;

pub async fn subscribe(
    caller: Caller<'_, PluginState>,
    topic_id: TopicId,
    fn_table_index: u32,
    priority: i32,
) {
    let data = caller.data();
    let store = data.store().clone();

    let mut handler_registry = data.host.handler_registry.write().await;
    handler_registry.subscribe(topic_id, store, fn_table_index, priority as i8);
}

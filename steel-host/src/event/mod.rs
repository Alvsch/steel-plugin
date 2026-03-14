use crate::event::handler::{HandlerEntry, HandlerRegistry};
use crate::plugin::PluginState;
use crate::utils;
use steel_plugin_sdk::event::TopicId;
use wasmtime::Store;

pub mod handler;
pub mod topic;

pub async fn dispatch_topic(handler_registry: &HandlerRegistry, topic_id: TopicId, payload: &[u8]) {
    let handlers = handler_registry.get_handlers(topic_id);
    for handler in handlers {
        let mut store = handler.store.lock().await;
        dispatch_event(&mut store, payload, handler).await;
    }
}

async fn dispatch_event(store: &mut Store<PluginState>, payload: &[u8], handler: &HandlerEntry) {
    let data = store.data();
    let exports = data.exports().clone();
    let instance = &exports.instance;
    let scratch = data.scratch;

    let fat_ptr = utils::write_scratch(store, exports.memory, &exports.alloc, scratch, payload)
        .await
        .unwrap();

    handler
        .handler_fn
        .call_async(&mut *store, fat_ptr.pack())
        .await
        .unwrap();

    utils::dealloc_scratch(store, instance, fat_ptr)
        .await
        .unwrap();
}

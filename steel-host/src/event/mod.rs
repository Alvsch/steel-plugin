use crate::event::handler::{HandlerFn, HandlerRegistry};
use crate::plugin::PluginState;
use crate::utils;
use crate::utils::memory::PluginMemory;
use steel_plugin_sdk::event::TopicId;
use steel_plugin_sdk::utils::fat::FatPtr;
use wasmtime::Store;

pub mod handler;

pub async fn dispatch_topic(
    handler_registry: &HandlerRegistry,
    topic_id: TopicId,
    payload: &mut Vec<u8>,
) {
    let handlers = handler_registry.get_handlers(topic_id);
    for handler in handlers {
        let mut store = handler.store.lock().await;
        dispatch_event(&mut store, payload, &handler.handler_fn).await;
    }
}

async fn dispatch_event(
    store: &mut Store<PluginState>,
    payload: &mut Vec<u8>,
    handler: &HandlerFn,
) {
    let data = store.data();
    let exports = data.exports().clone();
    let scratch = data.scratch;

    let fat_ptr = utils::write_scratch(store, exports.memory, &exports.alloc, scratch, payload)
        .await
        .unwrap();

    let result_ptr = FatPtr::unpack(
        handler
            .call_async(&mut *store, fat_ptr.pack())
            .await
            .unwrap(),
    );

    utils::dealloc_scratch(store, &exports.instance, fat_ptr)
        .await
        .unwrap();

    let Some(result) = result_ptr else {
        return;
    };

    let memory = PluginMemory::new(exports.memory, store);
    let value = memory.read(result).to_vec();
    exports
        .dealloc
        .call_async(store, (result.ptr(), result.len()))
        .await
        .unwrap();

    // TODO: validate returned event
    *payload = value;
}

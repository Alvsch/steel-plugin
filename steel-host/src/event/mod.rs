use crate::event::handler::{HandlerEntry, HandlerRegistry};
use crate::plugin::PluginState;
use crate::utils;
use steel_plugin_sdk::event::TopicId;
use wasmtime::{Ref, Store, TypedFunc};

pub mod handler;
pub mod topic;

pub async fn dispatch_topic(topic_id: TopicId, payload: &[u8], handler_registry: &HandlerRegistry) {
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

    let table = instance
        .get_table(&mut *store, "__indirect_function_table")
        .unwrap();

    let func_ref = table
        .get(&mut *store, u64::from(handler.fn_table_index))
        .unwrap();

    let Ref::Func(Some(func)) = func_ref else {
        utils::dealloc_scratch(store, instance, fat_ptr)
            .await
            .unwrap();
        return;
    };

    let typed: TypedFunc<u64, ()> = func.typed(&*store).unwrap();
    typed.call_async(&mut *store, fat_ptr.pack()).await.unwrap();

    utils::dealloc_scratch(store, instance, fat_ptr)
        .await
        .unwrap();
}

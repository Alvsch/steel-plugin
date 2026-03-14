use crate::plugin::PluginState;
use steel_plugin_sdk::event::TopicId;
use wasmtime::{Caller, TypedFunc};

pub async fn subscribe(
    mut caller: Caller<'_, PluginState>,
    topic_id: TopicId,
    fn_table_index: u32,
    priority: i32,
) {
    let data = caller.data();
    let store = data.store().clone();
    let instance = data.exports().clone().instance;

    let table = instance
        .get_table(&mut caller, "__indirect_function_table")
        .unwrap();

    let func_ref = table.get(&mut caller, u64::from(fn_table_index)).unwrap();

    let func = func_ref.as_func().unwrap().unwrap();
    let typed: TypedFunc<u64, ()> = func.typed(&mut caller).unwrap();

    let data = caller.data();
    let mut handler_registry = data.host.handler_registry.write().await;
    handler_registry.subscribe(topic_id, store, typed, priority as i8);
}

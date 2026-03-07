use steel_plugin_sdk::{
    rpc::{MethodId, PluginId},
    utils::fat::FatPtr,
};
use wasmtime::Caller;

use crate::{plugin::PluginState, utils, utils::memory::PluginMemory};

pub async fn register(mut caller: Caller<'_, PluginState>, export_name: u64) {
    let export_name = FatPtr::unpack(export_name).unwrap();
    let memory = PluginMemory::from(&mut caller);

    let export_name = memory.read(export_name);
    let export_name = str::from_utf8(export_name).unwrap().to_string();

    let export = caller
        .get_export(&export_name)
        .unwrap()
        .into_func()
        .unwrap()
        .typed::<u64, u64>(&mut caller)
        .unwrap();

    let plugin_id = caller.data().plugin_id;
    let mut rpc = caller.data().host.rpc.write().await;
    let method_id = rpc.next_id();
    rpc.get_plugin_mut(plugin_id)
        .unwrap()
        .register_method(method_id, export_name, export);
}

pub async fn resolve_plugin(mut caller: Caller<'_, PluginState>, plugin_name: u64) -> PluginId {
    let plugin_name_ptr = FatPtr::unpack(plugin_name).unwrap();
    let memory = PluginMemory::from(&mut caller);
    let plugin_name = memory.read_string(plugin_name_ptr);

    let rpc = caller.data().host.rpc.read().await;
    rpc.resolve_plugin(&plugin_name).unwrap()
}

pub async fn resolve_method(
    mut caller: Caller<'_, PluginState>,
    plugin_id: PluginId,
    method_name: u64,
) -> MethodId {
    let method_name_ptr = FatPtr::unpack(method_name).unwrap();
    let memory = PluginMemory::from(&mut caller);
    let method_name = memory.read_string(method_name_ptr);

    let rpc = caller.data().host.rpc.read().await;
    rpc.resolve_method(plugin_id, &method_name).unwrap()
}

pub async fn dispatch(
    mut caller: Caller<'_, PluginState>,
    plugin_id: PluginId,
    method_id: MethodId,
    data_ptr: u64,
) -> u64 {
    let data_ptr = FatPtr::unpack(data_ptr).unwrap();
    let caller_memory = PluginMemory::from(&mut caller);
    let data = caller_memory.read(data_ptr).to_vec();

    let rpc = caller.data().host.rpc.read().await;
    let provider = rpc.get_plugin(plugin_id).unwrap();
    let method = provider.get_method(method_id);
    let mut provider_store = provider.store.lock().await;

    let memory = provider_store
        .data()
        .instance()
        .get_memory(&mut *provider_store, "memory")
        .unwrap();

    let scratch = provider_store.data().scratch;

    let fat_data =
        utils::write_scratch(&mut provider_store, memory, &provider.alloc, scratch, &data)
            .await
            .unwrap();

    let result = method
        .call_async(&mut *provider_store, fat_data.pack())
        .await
        .unwrap();

    let provider_instance = provider_store.data().instance();
    utils::dealloc_scratch(&mut provider_store, &provider_instance, fat_data)
        .await
        .unwrap();

    let Some(fat_result) = FatPtr::unpack(result) else {
        return 0;
    };

    // Read result from provider
    let provider_memory = PluginMemory::new(memory, &mut *provider_store);
    let data = provider_memory.read(fat_result).to_vec();

    drop(provider_store);
    drop(rpc);
    // Allocate result into caller
    let alloc = caller
        .data()
        .instance()
        .get_typed_func::<u32, u32>(&mut caller, "alloc")
        .unwrap();
    let ptr = alloc
        .call_async(&mut caller, fat_result.len())
        .await
        .unwrap();

    let mut caller_memory = PluginMemory::from(&mut caller);
    caller_memory.write(ptr, &data);

    FatPtr::new(ptr, fat_result.len()).unwrap().pack()
}

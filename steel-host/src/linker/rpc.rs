use steel_plugin_sdk::{
    rpc::{MethodId, PluginId},
    utils::fat::FatPtr,
};
use wasmtime::Caller;

use crate::{
    error::PluginContractError,
    plugin::PluginState,
    utils::{self, memory::PluginMemory},
};

pub async fn resolve_plugin(
    mut caller: Caller<'_, PluginState>,
    plugin_name: FatPtr,
) -> Result<Option<PluginId>, PluginContractError> {
    let exports = caller.data().exports().clone();
    let memory = PluginMemory::new(&mut caller, &exports.memory);
    let plugin_name = memory
        .read_string(plugin_name)
        .map_err(|err| PluginContractError::Other(err.to_string()))?;

    let plugin_id = caller.data().host.resolve_plugin(&plugin_name).await;

    Ok(plugin_id)
}

pub async fn resolve_method(
    mut caller: Caller<'_, PluginState>,
    plugin_id: PluginId,
    method_name: FatPtr,
) -> Result<Option<MethodId>, PluginContractError> {
    let exports = caller.data().exports().clone();
    let memory = PluginMemory::new(&mut caller, &exports.memory);
    let method_name = memory
        .read_string(method_name)
        .map_err(|err| PluginContractError::Other(err.to_string()))?;

    let rpc = caller.data().host.rpc.read().await;
    let method_id = rpc.resolve_method(plugin_id, &method_name);
    Ok(method_id)
}

pub async fn dispatch(
    mut caller: Caller<'_, PluginState>,
    plugin_id: PluginId,
    method_id: MethodId,
    data_ptr: FatPtr,
) -> Result<Option<FatPtr>, PluginContractError> {
    let caller_exports = caller.data().exports().clone();
    let caller_memory = PluginMemory::new(&mut caller, &caller_exports.memory);
    let data = caller_memory.read(data_ptr).to_vec();

    let rpc = caller.data().host.rpc.read().await;
    let provider = rpc
        .get_plugin(plugin_id)
        .ok_or(PluginContractError::InvalidId)?;
    let method = provider
        .get_method(method_id)
        .ok_or(PluginContractError::InvalidId)?;

    let mut provider_store = provider.store.lock().await;
    let provider_data = provider_store.data();
    let provider_exports = provider_data.exports().clone();
    let provider_scratch = provider_data.scratch;

    let fat_data = utils::write_scratch(
        &mut provider_store,
        provider_exports.memory,
        &provider_exports,
        provider_scratch,
        &data,
    )
    .await?;

    let result = method
        .call_async(&mut *provider_store, fat_data.pack())
        .await?;

    utils::dealloc_scratch(&mut provider_store, &provider_exports.instance, fat_data).await?;

    let Some(fat_result) = FatPtr::unpack(result) else {
        return Ok(None);
    };

    // Read result from provider
    let provider_memory = PluginMemory::new(&mut *provider_store, &provider_exports.memory);
    let data = provider_memory.read(fat_result).to_vec();

    drop(provider_store);
    drop(rpc);

    // Allocate result into caller
    let fat = caller_exports.alloc(&mut caller, fat_result.len()).await?;

    let mut caller_memory = PluginMemory::new(&mut caller, &caller_exports.memory);
    caller_memory.write(fat.ptr(), &data);

    Ok(Some(fat))
}

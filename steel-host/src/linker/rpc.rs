use steel_plugin_sdk::{
    rpc::{MethodId, PluginId},
    utils::fat::FatPtr,
};
use wasmtime::Caller;

use crate::{
    error::PluginContractError,
    plugin::PluginState,
    utils::{self, memory::MemoryExt},
};

pub async fn resolve_plugin(
    caller: Caller<'_, PluginState>,
    plugin_name: FatPtr,
) -> Result<Option<PluginId>, PluginContractError> {
    let exports = caller.data().exports().clone();
    let plugin_name = exports
        .memory
        .read_string(&caller, plugin_name)
        .map_err(|err| PluginContractError::Other(err.to_string()))?;

    let plugin_id = caller.data().host.resolve_plugin(&plugin_name).await;

    Ok(plugin_id)
}

pub async fn resolve_method(
    caller: Caller<'_, PluginState>,
    plugin_id: PluginId,
    method_name: FatPtr,
) -> Result<Option<MethodId>, PluginContractError> {
    let exports = caller.data().exports().clone();
    let method_name = exports
        .memory
        .read_string(&caller, method_name)
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
    let data = caller_exports
        .memory
        .read_memory(&caller, data_ptr)
        .to_vec();

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
    let data = provider_exports
        .memory
        .read_memory(&*provider_store, fat_result)
        .to_vec();

    drop(provider_store);
    drop(rpc);

    // Allocate result into caller
    let fat = caller_exports.alloc(&mut caller, fat_result.len()).await?;
    caller_exports
        .memory
        .write_memory(&mut caller, fat.ptr(), &data);

    Ok(Some(fat))
}

use crate::plugin::PluginState;
use crate::utils;
use crate::{error::PluginContractError, utils::memory::MemoryExt};
use steel_plugin_sdk::utils::fat::FatPtr;
use wasmtime::Store;

mod handler;
pub use handler::{HandlerFn, HandlerRegistry};

async fn dispatch_event(
    store: &mut Store<PluginState>,
    payload: &mut Vec<u8>,
    handler: &HandlerFn,
) -> Result<(), PluginContractError> {
    let data = store.data();
    let exports = data.exports().clone();
    let scratch = data.scratch;

    let fat = utils::write_scratch(store, exports.memory, &exports, scratch, payload).await?;

    let result_ptr = FatPtr::unpack(handler.call_async(&mut *store, fat.pack()).await?);

    utils::dealloc_scratch(store, &exports.instance, fat).await?;

    let Some(result) = result_ptr else {
        return Ok(());
    };

    let value = exports.memory.read_memory(store, result).to_vec();
    exports.dealloc(store, result).await?;

    // TODO: validate returned event
    *payload = value;
    Ok(())
}

use crate::error::PluginContractError;
use crate::plugin::PluginState;
use crate::utils;
use wasmtime::Store;

mod handler;
pub use handler::{HandlerFn, HandlerRegistry};

async fn dispatch_event(
    store: &mut Store<PluginState>,
    payload: &[u8],
    handler: &HandlerFn,
) -> Result<(), PluginContractError> {
    let data = store.data();
    let exports = data.exports().clone();
    let scratch = data.scratch;

    let fat = utils::write_scratch(store, exports.memory, &exports, scratch, payload).await?;

    handler.call_async(&mut *store, fat.pack()).await?;
    utils::dealloc_scratch(store, &exports.instance, fat).await?;
    Ok(())
}

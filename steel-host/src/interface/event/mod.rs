use crate::{Plugin, error::PluginContractError};

mod handler;
pub use handler::{HandlerFn, HandlerRegistry};

async fn dispatch_event(
    plugin: &Plugin,
    payload: &[u8],
    handler: HandlerFn,
) -> Result<(), PluginContractError> {
    plugin
        .bindings
        .lock()
        .await
        .host_plugin_sdk_plugin_api()
        .call_event_handler(&mut *plugin.store.lock().await, handler, payload)
        .await?;

    Ok(())
}

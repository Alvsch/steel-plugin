use crate::{Plugin, error::PluginContractError};

mod handler;
pub use handler::{HandlerFn, HandlerRegistry};

fn dispatch_event(
    plugin: &Plugin,
    payload: &[u8],
    handler: HandlerFn,
) -> Result<(), PluginContractError> {
    plugin
        .bindings
        .lock()
        .host_plugin_sdk_plugin_api()
        .call_event_handler(&mut *plugin.store.lock(), handler, payload)?;

    Ok(())
}

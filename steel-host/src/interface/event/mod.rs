use crate::{
    Plugin, error::PluginContractError, linker::exports::host::plugin_sdk::plugin_api::Event,
};

mod handler;
pub use handler::{HandlerFn, HandlerRegistry};

async fn dispatch_event(
    plugin: &Plugin,
    event: &mut Event,
    handler: HandlerFn,
) -> Result<(), PluginContractError> {
    todo!();
    // plugin
    //     .bindings
    //     .lock()
    //     .await
    //     .host_plugin_sdk_plugin_api()
    //     .call_event_handler(&mut *plugin.store.lock().await, handler, event)
    //     .await
    //     .unwrap();

    // Ok(())
}

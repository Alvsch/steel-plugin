use crate::{
    error::PluginContractError,
    linker::{
        host::plugin_sdk::event::{Event as WasmEvent, PlayerJoinEvent, PlayerLeaveEvent},
        player::PlayerResource,
    },
    plugin::Plugin,
};

use steel_core::PluginApi;
use wasmtime::component::Resource;

pub use handler::{HandlerFn, HandlerRegistry};
mod handler;

pub(super) async fn api_to_wasm_event(
    api: PluginApi,
    plugin: &Plugin,
) -> wasmtime::Result<WasmEvent> {
    let mut store = plugin.store.lock().await;
    let data = store.data_mut();
    match api {
        PluginApi::PlayerJoinEvent(player) => {
            let wasm_player = data.table.push(PlayerResource { provider: player })?;
            Ok(WasmEvent::PlayerJoinEvent(PlayerJoinEvent {
                player: Resource::new_own(wasm_player.rep()),
            }))
        }
        PluginApi::PlayerLeaveEvent(player) => {
            let wasm_player = data.table.push(PlayerResource { provider: player })?;
            Ok(WasmEvent::PlayerLeaveEvent(PlayerLeaveEvent {
                player: Resource::new_own(wasm_player.rep()),
            }))
        }
    }
}

async fn dispatch_event(
    plugin: &Plugin,
    event: WasmEvent,
    handler: HandlerFn,
) -> Result<(), PluginContractError> {
    plugin
        .bindings
        .lock()
        .await
        .host_plugin_sdk_plugin_api()
        .call_event_handler(&mut *plugin.store.lock().await, handler, event)
        .await?;

    Ok(())
}

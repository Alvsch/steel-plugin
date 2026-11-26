mod handler;

use crate::{
    error::PluginContractError,
    linker::host::plugin_sdk::event::{self, Event, PlayerJoinEvent, PlayerLeaveEvent},
    plugin::{Plugin, PluginState, PluginStore},
};

use steel_core::PluginApi;
use steel_plugin_core::{TopicId, fnv1a_32};

pub(crate) use handler::{HandlerFn, HandlerRegistry};

impl event::Host for PluginState {
    async fn register_event(&mut self, handler_id: u32, topic_id: u32, priority: i32) {
        self.host.handler_registry.write().await.subscribe(
            TopicId(topic_id),
            self.meta.name.clone(),
            handler_id,
            priority as i8,
        );
    }
}

pub(crate) trait WasmEvent {
    fn topic_id(&self) -> TopicId;
    fn into_wasm(self, store: &mut PluginStore) -> wasmtime::Result<Event>;
}

impl WasmEvent for PluginApi {
    fn topic_id(&self) -> TopicId {
        TopicId(match self {
            PluginApi::PlayerJoinEvent(_) => fnv1a_32(b"PlayerJoinEvent"),
            PluginApi::PlayerLeaveEvent(_) => fnv1a_32(b"PlayerLeaveEvent"),
        })
    }

    fn into_wasm(self, store: &mut PluginStore) -> wasmtime::Result<Event> {
        let data = store.data_mut();
        match self {
            PluginApi::PlayerJoinEvent(player) => {
                let wasm_player = data.resources.push_player(player)?;
                Ok(Event::PlayerJoinEvent(PlayerJoinEvent {
                    player: wasm_player,
                }))
            }
            PluginApi::PlayerLeaveEvent(player) => {
                let wasm_player = data.resources.push_player(player)?;
                Ok(Event::PlayerLeaveEvent(PlayerLeaveEvent {
                    player: wasm_player,
                }))
            }
        }
    }
}

async fn dispatch_event<E: WasmEvent>(
    plugin: &Plugin,
    event: E,
    handler: HandlerFn,
) -> Result<(), PluginContractError> {
    let mut runtime = plugin.runtime.lock().await;
    let crate::plugin::PluginRuntime { store, bindings } = &mut *runtime;
    let event = event.into_wasm(store)?;
    bindings
        .host_plugin_sdk_plugin_api()
        .call_event_handler(&mut *store, handler, event)
        .await?;

    Ok(())
}

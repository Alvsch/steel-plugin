use steel_plugin_sdk::objects::HandleKey;
use tracing::warn;
use wasmtime::component::Linker;

use crate::host::plugin_sdk;
use crate::interface::objects::{BatchDispatchOutcome, FetchOutcome};
use crate::plugin::PluginState;

pub type HostLinker = Linker<PluginState>;

impl plugin_sdk::logging::Host for PluginState {
    fn error(&mut self, message: String) {
        tracing::error!("[{}] {message}", self.meta.name);
    }

    fn warn(&mut self, message: String) {
        tracing::warn!("[{}] {message}", self.meta.name);
    }

    fn info(&mut self, message: String) {
        tracing::info!("[{}] {message}", self.meta.name);
    }

    fn debug(&mut self, message: String) {
        tracing::debug!("[{}] {message}", self.meta.name);
    }

    fn trace(&mut self, message: String) {
        tracing::trace!("[{}] {message}", self.meta.name);
    }
}

impl plugin_sdk::rpc::Host for PluginState {
    fn resolve_plugin(&mut self, name: String) -> Option<u32> {
        self.host.resolve_plugin(&name)
    }

    fn resolve_method(&mut self, plugin_id: u32, method_name: String) -> Option<u32> {
        self.host.rpc.read().resolve_method(plugin_id, &method_name)
    }

    fn dispatch(&mut self, plugin_id: u32, method_id: u32, data: Vec<u8>) -> Option<Vec<u8>> {
        let rpc = self.host.rpc.read();
        let plugin = rpc.get_plugin(plugin_id).expect("invalid plugin");
        let mut store = plugin.store.store.lock();

        plugin
            .store
            .bindings
            .lock()
            .host_plugin_sdk_plugin_api()
            .call_rpc(&mut *store, method_id, &data)
            .expect("failed to call rpc")
    }
}

impl plugin_sdk::object::Host for PluginState {
    fn object_fetch(&mut self, entity_key: u64, queries: Vec<u8>) -> Option<Vec<u8>> {
        let outcome = {
            let objects = self.host.objects.read();
            objects.fetch(HandleKey::from_ffi(entity_key), &queries)
        };

        match outcome {
            FetchOutcome::MissingKey => {
                warn!(
                    entity_key = entity_key,
                    "object_fetch called with unknown handle key"
                );
                None
            }
            FetchOutcome::HandlerError(err) => {
                warn!(entity_key = entity_key, error = %err, "object_fetch handler failed");
                None
            }
            FetchOutcome::Response(response) => Some(response),
        }
    }

    fn object_batch_dispatch(&mut self, entity_key: u64, commands: Vec<u8>) {
        let outcome = {
            let host = self.host.clone();
            let objects = host.objects.read();
            objects.batch_dispatch(HandleKey::from_ffi(entity_key), &commands)
        };

        match outcome {
            BatchDispatchOutcome::Dispatched => (),
            BatchDispatchOutcome::MissingKey => {
                warn!(
                    entity_key = entity_key,
                    "object_batch_dispatch called with unknown handle key"
                );
            }
            BatchDispatchOutcome::HandlerError(err) => {
                warn!(entity_key = entity_key, error = %err, "object_batch_dispatch handler failed");
            }
        }
    }
}

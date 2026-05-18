use steel_plugin_sdk::rpc::PluginId;
use wasmtime::component::{HasSelf, Linker, Resource};

use crate::{linker::host::plugin_sdk::{self, player::Player}, plugin::PluginState};

pub type HostLinker = Linker<PluginState>;

wasmtime::component::bindgen!({
    path: "../wit",
    exports: { default: async | trappable },
    imports: { default: async },
});

pub fn add_to_linker(linker: &mut HostLinker) -> wasmtime::Result<()> {
    plugin_sdk::logging::add_to_linker::<_, HasSelf<_>>(linker, |state| state)?;
    plugin_sdk::player::add_to_linker::<_, HasSelf<_>>(linker, |state| state)?;
    plugin_sdk::rpc::add_to_linker::<_, HasSelf<_>>(linker, |state| state)?;
    Ok(())
}

impl plugin_sdk::logging::Host for PluginState {
    async fn error(&mut self, message: String) {
        tracing::error!("[{}] {message}", self.meta.name);
    }

    async fn warn(&mut self, message: String) {
        tracing::warn!("[{}] {message}", self.meta.name);
    }

    async fn info(&mut self, message: String) {
        tracing::info!("[{}] {message}", self.meta.name);
    }

    async fn debug(&mut self, message: String) {
        tracing::debug!("[{}] {message}", self.meta.name);
    }

    async fn trace(&mut self, message: String) {
        tracing::trace!("[{}] {message}", self.meta.name);
    }
}

impl plugin_sdk::rpc::Host for PluginState {
    async fn resolve_plugin(&mut self, name: String) -> Option<u32> {
        self.host.resolve_plugin(&name).map(|x| x.0)
    }

    async fn dispatch(
        &mut self,
        plugin_id: u32,
        method_name: String,
        data: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let plugin = {
            let rpc = self.host.rpc.read();
            rpc.get_plugin(PluginId(plugin_id))
                .expect("invalid plugin")
                .clone()
        };
        let mut store = plugin.store.lock().await;

        plugin
            .bindings
            .lock()
            .await
            .host_plugin_sdk_plugin_api()
            .call_rpc(&mut *store, &method_name, &data)
            .await
            .expect("failed to call rpc")
    }
}

impl plugin_sdk::player::Host for PluginState {}

impl plugin_sdk::player::HostPlayer for PluginState {
    async fn get_health(&mut self, _player: Resource<Player>) -> u32 {
        20
    }

    async fn drop(&mut self, _player: Resource<Player>) -> wasmtime::Result<()> {
        Ok(())
    }
}

use crate::{
    linker::host::plugin_sdk::rpc::{self, PluginId},
    plugin::PluginState,
};

mod host;
pub use host::RpcRegistry;

impl rpc::Host for PluginState {
    async fn resolve_plugin(&mut self, name: String) -> Option<PluginId> {
        self.host.resolve_plugin(&name)
    }

    async fn dispatch(
        &mut self,
        plugin_id: PluginId,
        method_name: String,
        data: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let plugin = {
            let rpc = self.host.rpc.read();
            rpc.get_plugin(plugin_id).expect("invalid plugin").clone()
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

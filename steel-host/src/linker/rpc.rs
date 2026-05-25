use wasmtime::component::Resource;
use wasmtime_wasi::ResourceTableError;

use crate::{
    linker::host::plugin_sdk::rpc::{self, HostMethod},
    plugin::{PluginInstance, PluginResources, PluginState},
};

struct MethodResource {
    plugin: PluginInstance,
    name: String,
}

impl PluginResources {
    fn push_method(
        &mut self,
        plugin: PluginInstance,
        method_name: String,
    ) -> Result<Resource<rpc::Method>, ResourceTableError> {
        let resource = self.table.push(MethodResource {
            plugin,
            name: method_name,
        })?;
        Ok(Resource::new_own(resource.rep()))
    }

    fn get_method(
        &mut self,
        key: Resource<rpc::Method>,
    ) -> Result<&MethodResource, ResourceTableError> {
        let resource = self
            .table
            .get::<MethodResource>(&Resource::new_borrow(key.rep()))?;
        Ok(resource)
    }

    fn delete_method(
        &mut self,
        resource: Resource<rpc::Method>,
    ) -> Result<MethodResource, ResourceTableError> {
        debug_assert!(resource.owned());
        let resource = self
            .table
            .delete::<MethodResource>(Resource::new_own(resource.rep()))?;
        Ok(resource)
    }
}

impl rpc::Host for PluginState {
    async fn resolve_method(
        &mut self,
        plugin_name: String,
        method_name: String,
    ) -> Option<Resource<rpc::Method>> {
        let plugin = self.host.resolve_plugin(&plugin_name)?;
        let method = self
            .resources
            .push_method(plugin.clone(), method_name)
            .expect("failed to push resource");
        Some(method)
    }
}
impl HostMethod for PluginState {
    async fn dispatch(
        &mut self,
        resource: Resource<rpc::Method>,
        data: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let method = self
            .resources
            .get_method(resource)
            .expect("unknown resource");

        let mut store = method.plugin.store.lock().await;
        method
            .plugin
            .bindings
            .lock()
            .await
            .host_plugin_sdk_plugin_api()
            .call_rpc(&mut *store, &method.name, &data)
            .await
            .expect("failed to run rpc")
    }

    async fn drop(&mut self, resource: Resource<rpc::Method>) -> wasmtime::Result<()> {
        if resource.owned() {
            self.resources.delete_method(resource)?;
        }
        Ok(())
    }
}

use std::sync::Arc;

use steel_core::{entity::LivingEntity, player::Player};
use wasmtime::component::Resource;
use wasmtime_wasi::ResourceTableError;

use crate::{
    linker::host::plugin_sdk::player,
    plugin::{PluginResources, PluginState},
};

struct PlayerResource {
    pub provider: Arc<Player>,
}

impl PluginResources {
    pub fn push_player(
        &mut self,
        provider: Arc<Player>,
    ) -> Result<Resource<player::Player>, ResourceTableError> {
        let resource = self.table.push(PlayerResource { provider })?;
        Ok(Resource::new_own(resource.rep()))
    }

    pub fn get_player(
        &mut self,
        key: Resource<player::Player>,
    ) -> Result<&Arc<Player>, ResourceTableError> {
        let resource = self
            .table
            .get::<PlayerResource>(&Resource::new_borrow(key.rep()))?;
        Ok(&resource.provider)
    }

    pub fn delete_player(
        &mut self,
        resource: Resource<player::Player>,
    ) -> Result<Arc<Player>, ResourceTableError> {
        if !resource.owned() {
            tracing::warn!("deleting a borrowed resource");
        }
        let resource = self
            .table
            .delete::<PlayerResource>(Resource::new_borrow(resource.rep()))?;
        Ok(resource.provider)
    }
}

impl player::Host for PluginState {}

impl player::HostPlayer for PluginState {
    async fn get_health(&mut self, res: Resource<player::Player>) -> f32 {
        let player = self
            .resources
            .table
            .get::<PlayerResource>(&Resource::new_borrow(res.rep()))
            .expect("failed to access resource");
        player.provider.get_health()
    }

    async fn drop(&mut self, res: Resource<player::Player>) -> wasmtime::Result<()> {
        if res.owned() {
            self.resources.delete_player(res)?;
        }
        Ok(())
    }
}

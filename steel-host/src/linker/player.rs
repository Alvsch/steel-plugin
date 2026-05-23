use std::sync::Arc;

use steel_core::{entity::LivingEntity, player::Player};
use wasmtime::component::Resource;

use crate::{linker::host::plugin_sdk::player, plugin::PluginState};

pub struct PlayerResource {
    pub provider: Arc<Player>,
}

impl player::Host for PluginState {}

impl player::HostPlayer for PluginState {
    async fn get_health(&mut self, res: Resource<player::Player>) -> f32 {
        let player = self
            .table
            .get::<PlayerResource>(&Resource::new_borrow(res.rep()))
            .expect("failed to access resource");
        player.provider.get_health()
    }

    async fn drop(&mut self, res: Resource<player::Player>) -> wasmtime::Result<()> {
        if res.owned() {
            self.table
                .delete::<PlayerResource>(Resource::new_own(res.rep()))?;
        }
        Ok(())
    }
}

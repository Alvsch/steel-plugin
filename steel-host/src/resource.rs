use std::sync::Arc;

use steel_core::player::Player;

pub struct PlayerResource {
    pub provider: Arc<Player>,
}

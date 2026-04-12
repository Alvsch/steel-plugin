use serde::Serialize;
use std::sync::Arc;
use steel_core::{
    entity::{Entity, LivingEntity},
    player::Player,
};
use steel_plugin_sdk::objects::{
    self,
    player::{PlayerCommand, PlayerQuery},
};
use steel_utils::types::GameType;

use super::ObjectHandler;

pub fn make_player_handler(player: Arc<Player>) -> ObjectHandler {
    let fetch_state = Arc::clone(&player);
    let batch_state = Arc::clone(&player);

    ObjectHandler::make::<objects::player::Player, _, _>(
        move |serializer, query| {
            let state = fetch_state.as_ref();
            match query {
                PlayerQuery::Name => state.gameprofile.name.serialize(serializer),
                PlayerQuery::Position => state.position().serialize(serializer),
                PlayerQuery::Gamemode => state.game_mode.load().serialize(serializer),
                PlayerQuery::Health => state.get_health().serialize(serializer),
            }
            .expect("failed to serialize");
            Ok(())
        },
        move |command| {
            let state = batch_state.as_ref();
            match command {
                PlayerCommand::SendMessage(message) => {
                    state.send_message(&message);
                }
                PlayerCommand::SetGamemode(gamemode) => {
                    state.set_game_mode(GameType::from(gamemode));
                }
                PlayerCommand::SetHealth(health) => {
                    state.set_health(health);
                }
                PlayerCommand::Kick(reason) => {
                    state.disconnect(reason);
                }
                PlayerCommand::Teleport(position) => {
                    let (yaw, pitch) = state.rotation();
                    state.teleport(position.x, position.y, position.z, yaw, pitch);
                }
            }
            Ok(())
        },
    )
}

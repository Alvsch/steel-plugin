use rmp::encode::write_array_len;
use serde::Serialize;
use std::sync::Arc;
use steel_core::{
    entity::{Entity, LivingEntity},
    player::Player,
};
use steel_plugin_sdk::objects::player::{PlayerCommand, PlayerQuery};
use steel_utils::types::GameType;

use super::ObjectHandler;

fn encode_player_fetch_response(
    queries: Vec<PlayerQuery>,
    state: &Player,
) -> Result<Vec<u8>, String> {
    let query_count = u32::try_from(queries.len())
        .map_err(|_| "too many player queries in a single fetch".to_string())?;

    let mut payload = Vec::new();
    write_array_len(&mut payload, query_count)
        .map_err(|err| format!("failed to encode response array header: {err}"))?;

    let mut serializer = rmp_serde::Serializer::new(&mut payload);
    for query in queries {
        match query {
            PlayerQuery::Name => state.gameprofile.name.serialize(&mut serializer),
            PlayerQuery::Position => state.position().serialize(&mut serializer),
            PlayerQuery::Gamemode => state.game_mode.load().serialize(&mut serializer),
            PlayerQuery::Health => state.get_health().serialize(&mut serializer),
        }
        .map_err(|err| format!("failed to serialize fetch response value: {err}"))?;
    }

    Ok(payload)
}

fn apply_player_commands(state: &Player, commands: Vec<PlayerCommand>) {
    for cmd in commands {
        match cmd {
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
    }
}

pub fn make_player_handler(player: Arc<Player>) -> ObjectHandler {
    let fetch_state = Arc::clone(&player);
    let batch_state = Arc::clone(&player);

    ObjectHandler::from_fns(
        move |payload| {
            let queries: Vec<PlayerQuery> = rmp_serde::from_slice(payload)
                .map_err(|err| format!("failed to decode player queries: {err}"))?;

            encode_player_fetch_response(queries, &fetch_state)
        },
        move |payload| {
            let commands: Vec<PlayerCommand> = rmp_serde::from_slice(payload)
                .map_err(|err| format!("failed to decode player commands: {err}"))?;

            apply_player_commands(&batch_state, commands);
            Ok(())
        },
    )
}

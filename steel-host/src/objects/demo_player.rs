use glam::DVec3;
use rmp::encode::write_array_len;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use steel_plugin_sdk::objects::{
    GameType,
    player::{PlayerCommand, PlayerQuery},
};
use tracing::info;

use super::ObjectHandler;

#[derive(Debug)]
pub struct DemoPlayer {
    pub name: String,
    pub health: f32,
    pub position: DVec3,
    pub gamemode: GameType,
}

fn encode_player_fetch_response(
    queries: Vec<PlayerQuery>,
    state: &DemoPlayer,
) -> Result<Vec<u8>, String> {
    let query_count = u32::try_from(queries.len())
        .map_err(|_| "too many player queries in a single fetch".to_string())?;

    let mut payload = Vec::new();
    write_array_len(&mut payload, query_count)
        .map_err(|err| format!("failed to encode response array header: {err}"))?;

    let mut serializer = rmp_serde::Serializer::new(&mut payload);
    for query in queries {
        match query {
            PlayerQuery::Name => state.name.clone().serialize(&mut serializer),
            PlayerQuery::Position => state.position.serialize(&mut serializer),
            PlayerQuery::Gamemode => state.gamemode.serialize(&mut serializer),
            PlayerQuery::Health => state.health.serialize(&mut serializer),
        }
        .map_err(|err| format!("failed to serialize fetch response value: {err}"))?;
    }

    Ok(payload)
}

fn apply_player_commands(state: &mut DemoPlayer, commands: Vec<PlayerCommand>) {
    for cmd in commands {
        match cmd {
            PlayerCommand::SendMessage(message) => {
                info!("[demo-player:{}] chat: {message}", state.name);
            }
            PlayerCommand::SetGamemode(gamemode) => {
                state.gamemode = gamemode;
                info!(
                    "[demo-player:{}] gamemode set to {:?}",
                    state.name, state.gamemode
                );
            }
            PlayerCommand::SetPosition(position) => {
                state.position = position;
                info!(
                    "[demo-player:{}] position set to {:?}",
                    state.name, state.position
                );
            }
            PlayerCommand::SetHealth(health) => {
                state.health = health.clamp(0.0, 20.0);
                info!(
                    "[demo-player:{}] health set to {}",
                    state.name, state.health
                );
            }
            PlayerCommand::Kick(reason) => {
                info!("[demo-player:{}] kick requested: {reason}", state.name);
            }
            PlayerCommand::Teleport { x, y, z } => {
                state.position = DVec3::new(x, y, z);
                info!(
                    "[demo-player:{}] teleported to {:?}",
                    state.name, state.position
                );
            }
        }
    }
}

pub fn make_player_handler(player: Arc<Mutex<DemoPlayer>>) -> ObjectHandler {
    let fetch_state = Arc::clone(&player);
    let batch_state = Arc::clone(&player);

    ObjectHandler::from_fns(
        move |payload| {
            let queries: Vec<PlayerQuery> = rmp_serde::from_slice(payload)
                .map_err(|err| format!("failed to decode player queries: {err}"))?;

            let state = fetch_state
                .lock()
                .map_err(|err| format!("demo player lock poisoned: {err}"))?;

            encode_player_fetch_response(queries, &state)
        },
        move |payload| {
            let commands: Vec<PlayerCommand> = rmp_serde::from_slice(payload)
                .map_err(|err| format!("failed to decode player commands: {err}"))?;

            let mut state = batch_state
                .lock()
                .map_err(|err| format!("demo player lock poisoned: {err}"))?;

            apply_player_commands(&mut state, commands);
            Ok(())
        },
    )
}

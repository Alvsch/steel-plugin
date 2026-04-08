use glam::DVec3;
use serde::{Deserialize, Serialize};

use crate::objects::{Entity, GameType, batch::BatchBuilder, query::QueryItem};

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerQuery {
    Name,
    Position,
    Gamemode,
    Health,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerCommand {
    SendMessage(String),
    SetGamemode(GameType),
    SetPosition(DVec3),
    SetHealth(f32),
    Kick(String),
    Teleport { x: f64, y: f64, z: f64 },
}

#[derive(Debug)]
pub struct Player;

impl Entity for Player {
    type WireQuery = PlayerQuery;
    type WireCommand = PlayerCommand;
}

pub struct Name;
pub struct Position;
pub struct Gamemode;
pub struct Health;

impl QueryItem<Player> for Name {
    type Output = String;

    fn to_wire() -> <Player as Entity>::WireQuery {
        PlayerQuery::Name
    }
}

impl QueryItem<Player> for Position {
    type Output = DVec3;

    fn to_wire() -> <Player as Entity>::WireQuery {
        PlayerQuery::Position
    }
}

impl QueryItem<Player> for Gamemode {
    type Output = GameType;

    fn to_wire() -> <Player as Entity>::WireQuery {
        PlayerQuery::Gamemode
    }
}

impl QueryItem<Player> for Health {
    type Output = f32;

    fn to_wire() -> <Player as Entity>::WireQuery {
        PlayerQuery::Health
    }
}

pub type PlayerBatch = BatchBuilder<Player>;

impl PlayerBatch {
    pub fn send_message(self, message: String) -> Self {
        self.push(PlayerCommand::SendMessage(message))
    }

    pub fn set_gamemode(self, gamemode: GameType) -> Self {
        self.push(PlayerCommand::SetGamemode(gamemode))
    }

    pub fn set_position(self, position: DVec3) -> Self {
        self.push(PlayerCommand::SetPosition(position))
    }

    pub fn set_health(self, health: f32) -> Self {
        self.push(PlayerCommand::SetHealth(health))
    }

    pub fn kick(self, reason: String) -> Self {
        self.push(PlayerCommand::Kick(reason))
    }
}

use glam::DVec3;
use serde::{Deserialize, Serialize};
use text_components::TextComponent;

use crate::objects::{Entity, batch::BatchBuilder, query::QueryItem};

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerQuery {
    Name,
    Position,
    Gamemode,
    Health,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerCommand {
    SendMessage(TextComponent),
    SetGamemode(i8),
    SetHealth(f32),
    Kick(TextComponent),
    Teleport(DVec3),
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
    type Output = i8;

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
    pub fn send_message(self, message: impl Into<TextComponent>) -> Self {
        self.push(PlayerCommand::SendMessage(message.into()))
    }

    pub fn set_gamemode(self, gamemode: i8) -> Self {
        self.push(PlayerCommand::SetGamemode(gamemode))
    }

    pub fn set_health(self, health: f32) -> Self {
        self.push(PlayerCommand::SetHealth(health))
    }

    pub fn kick(self, reason: impl Into<TextComponent>) -> Self {
        self.push(PlayerCommand::Kick(reason.into()))
    }

    pub fn teleport(self, position: DVec3) -> Self {
        self.push(PlayerCommand::Teleport(position))
    }
}

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::BlockPos;

pub mod handler;
pub mod result;

bitflags! {
    #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct EventHandlerFlags: u8 {
        const RECEIVE_CANCELLED = 1;
    }

}

pub trait Event: Serialize + for<'a> Deserialize<'a> {
    const NAME: &str;

    fn cancelled(&self) -> bool;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerJoinEvent {
    pub cancelled: bool,
    pub player: Uuid,
}

impl Event for PlayerJoinEvent {
    const NAME: &str = "PlayerJoinEvent";

    fn cancelled(&self) -> bool {
        self.cancelled
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerLeaveEvent {
    pub player: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerChatEvent {
    pub player: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockBreakEvent {
    pub player: Uuid,
    pub position: BlockPos,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockPlaceEvent {
    pub player: Uuid,
    pub position: BlockPos,
}

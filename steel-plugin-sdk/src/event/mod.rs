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

impl Event for PlayerLeaveEvent {
    const NAME: &str = "PlayerLeaveEvent";

    fn cancelled(&self) -> bool {
        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerChatEvent {
    pub cancelled: bool,
    pub player: Uuid,
    pub message: String,
}

impl Event for PlayerChatEvent {
    const NAME: &str = "PlayerChatEvent";

    fn cancelled(&self) -> bool {
        self.cancelled
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockBreakEvent {
    pub cancelled: bool,
    pub player: Uuid,
    pub position: BlockPos,
}

impl Event for BlockBreakEvent {
    const NAME: &str = "BlockBreakEvent";

    fn cancelled(&self) -> bool {
        self.cancelled
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockPlaceEvent {
    pub cancelled: bool,
    pub player: Uuid,
    pub position: BlockPos,
}

impl Event for BlockPlaceEvent {
    const NAME: &str = "BlockPlaceEvent";

    fn cancelled(&self) -> bool {
        self.cancelled
    }
}

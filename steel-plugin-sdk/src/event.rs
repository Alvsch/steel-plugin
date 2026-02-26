use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::FromRepr;
use uuid::Uuid;

use crate::types::BlockPos;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct EventHandlerFlags: u8 {
        const RECEIVE_CANCELLED = 1;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct EventResult: u8 {
        const MODIFIED = 1;
        const CANCELLED = 1 << 1;
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromRepr)]
pub enum EventId {
    PlayerJoinEvent,
    PlayerLeaveEvent,
    PlayerChatEvent,
    BlockBreakEvent,
    BlockPlaceEvent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerJoinEvent {
    pub player: Uuid,
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

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::FromRepr;
use uuid::Uuid;

use crate::types::BlockPos;

pub mod result;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct EventHandlerFlags: u8 {
        const RECEIVE_CANCELLED = 1;
    }
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub enum PluginEvent {
    PlayerJoinEvent(PlayerJoinEvent),
    PlayerLeaveEvent(PlayerLeaveEvent),
    PlayerChatEvent(PlayerChatEvent),
    BlockBreakEvent(BlockBreakEvent),
    BlockPlaceEvent(BlockPlaceEvent),
}

impl PluginEvent {
    pub fn event_id(&self) -> EventId {
        match self {
            PluginEvent::PlayerJoinEvent(_) => EventId::PlayerJoinEvent,
            PluginEvent::PlayerLeaveEvent(_) => EventId::PlayerLeaveEvent,
            PluginEvent::PlayerChatEvent(_) => EventId::PlayerChatEvent,
            PluginEvent::BlockBreakEvent(_) => EventId::BlockBreakEvent,
            PluginEvent::BlockPlaceEvent(_) => EventId::BlockPlaceEvent,
        }
    }
}

#[repr(u16)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, FromRepr)]
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

use serde::{Deserialize, Serialize};
use steel_plugin_core::TopicId;
use steel_plugin_macros::Event;
use uuid::Uuid;

pub trait Event: Serialize + for<'a> Deserialize<'a> {
    const TOPIC_ID: TopicId;
}

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct PlayerJoinEvent {
    pub player_id: Uuid,
    pub username: String,
}

use crate::host;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TopicId = u32;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerJoinEvent {
    pub player_id: Uuid,
    pub username: String,
}

#[must_use]
pub const fn hash_topic(bytes: &[u8]) -> TopicId {
    let mut hash: u32 = 0x811C_9DC5;
    let mut i = 0;

    while i < bytes.len() {
        hash ^= bytes[i] as u32;
        hash = hash.wrapping_mul(0x0100_0193);
        i += 1;
    }

    hash
}

pub fn event_subscribe(topic_id: TopicId, function: fn(u64) -> u64, priority: i8) {
    let fn_table_index = function as usize as u32;
    unsafe {
        host::event_subscribe(topic_id, fn_table_index, i32::from(priority));
    }
}

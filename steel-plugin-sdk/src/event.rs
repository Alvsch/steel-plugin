use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TopicId = u32;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerJoinEvent {
    pub player_id: Uuid,
    pub username: String,
}

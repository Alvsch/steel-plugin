use crate::host;
use serde::{Deserialize, Serialize};

pub type TopicId = u32;

#[derive(Serialize, Deserialize)]
pub struct PlayerJoinEvent {
    pub player_id: u64,
    pub username: String,
}

pub fn event_subscribe(topic_id: TopicId, function: fn(u64), priority: i8) {
    let fn_table_index = function as usize as u32;
    unsafe {
        host::event_subscribe(topic_id, fn_table_index, i32::from(priority));
    }
}

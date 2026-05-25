use inventory::collect;
use steel_plugin_core::TopicId;

use crate::sdk::event::Event;
pub use crate::sdk::event::{PlayerJoinEvent, PlayerLeaveEvent};

pub struct EventHandler {
    pub id: u32,
    pub topic_id: TopicId,
    pub priority: i8,
    pub function: fn(&mut Event),
}

collect!(EventHandler);

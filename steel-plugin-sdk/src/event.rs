use inventory::collect;

use crate::sdk::event::Event;
pub use crate::sdk::event::{PlayerJoinEvent, PlayerLeaveEvent};

pub struct EventHandler {
    pub id: u32,
    pub function: fn(&mut Event),
    pub priority: i8,
}

collect!(EventHandler);

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::event::EventHandlerFlags;

#[derive(Debug, Serialize, Deserialize)]
pub struct EventHandler {
    pub event_name: Cow<'static, str>,
    pub handler_name: Cow<'static, str>,
    pub priority: i8,
    pub flags: EventHandlerFlags,
}

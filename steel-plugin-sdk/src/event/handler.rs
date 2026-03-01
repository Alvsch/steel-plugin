use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::event::EventHandlerFlags;

/// Describes a handler registered for an event.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventHandler {
    /// Name of the event this handler listens to.
    pub event_name: Cow<'static, str>,
    /// Name of the handler function exported by the plugin.
    pub handler_name: Cow<'static, str>,
    /// Execution priority. Lower values run first.
    pub priority: i8,
    /// Flags that modify handler behavior.
    pub flags: EventHandlerFlags,
}

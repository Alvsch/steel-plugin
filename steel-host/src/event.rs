use std::collections::{BTreeMap, HashMap};

use steel_plugin_sdk::event::{EventHandlerFlags, EventId, result::EventResult};
use tokio::sync::Mutex;

use crate::PluginManager;

#[derive(Debug)]
struct EventHandler {
    plugin_name: String,
    // TODO: use these flags
    flags: EventHandlerFlags,
}

pub struct EventRegistry {
    registry: Mutex<HashMap<EventId, BTreeMap<i8, EventHandler>>>,
}

impl Default for EventRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: Mutex::new(HashMap::new()),
        }
    }

    pub async fn register_handler(
        &self,
        event_id: EventId,
        priority: i8,
        flags: EventHandlerFlags,
        plugin_name: String,
    ) {
        self.registry
            .lock()
            .await
            .entry(event_id)
            .or_default()
            .insert(priority, EventHandler { plugin_name, flags });
    }

    pub async fn call_event(
        &self,
        manager: &mut PluginManager,
        event_id: EventId,
        mut event: Vec<u8>,
    ) -> Vec<u8> {
        let lock = self.registry.lock().await;
        let Some(handlers) = lock.get(&event_id) else {
            return event;
        };
        let mut cancelled = false;
        for handler in handlers.values() {
            if cancelled && !handler.flags.contains(EventHandlerFlags::RECEIVE_CANCELLED) {
                continue;
            }

            let instance = manager.get_mut(&handler.plugin_name).unwrap();
            let result = instance.on_event(event_id, &event).await.unwrap();
            let result = EventResult::unpack(result).modified;

            if let Some((ptr, len)) = result {
                // TODO: index bounds?
                let (cancelled_data, data) = instance.memory.data(&mut instance.store)
                    [ptr as usize..(ptr + len) as usize]
                    .split_at(1);

                if !data.is_empty() {
                    // TODO: validation?
                    event = data.to_vec();
                }

                if cancelled_data[0] != 0 {
                    cancelled = true;
                }

                instance.dealloc(ptr, len).await.unwrap();
            }
        }
        event
    }
}

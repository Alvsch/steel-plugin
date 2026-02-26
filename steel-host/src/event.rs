use std::collections::{BTreeMap, HashMap};

use steel_plugin_sdk::event::{EventHandlerFlags, EventId};
use tokio::sync::Mutex;

use crate::PluginManager;

#[derive(Debug)]
struct EventHandler {
    plugin_name: String,
    // TODO: use these flags
    _flags: EventHandlerFlags,
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
            .insert(
                priority,
                EventHandler {
                    plugin_name,
                    _flags: flags,
                },
            );
    }

    pub async fn call_event(&self, manager: &mut PluginManager, event_id: EventId, event: Vec<u8>) {
        let lock = self.registry.lock().await;
        let Some(handlers) = lock.get(&event_id) else {
            return;
        };
        for handler in handlers.values() {
            let instance = manager.get_mut(&handler.plugin_name).unwrap();

            let len = event.len() as u32;
            let ptr = instance.write_to_memory(&event).await.unwrap();

            instance
                .exports
                .on_event
                .call_async(&mut instance.store, (event_id as u32, ptr, len))
                .await
                .unwrap();

            instance.dealloc(ptr, len).await.unwrap();
        }
    }
}

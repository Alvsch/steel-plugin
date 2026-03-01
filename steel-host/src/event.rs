use std::collections::{BTreeMap, HashMap};

use steel_plugin_sdk::event::{Event, EventHandlerFlags, handler::EventHandler};
use tokio::sync::Mutex;

use crate::PluginManager;

type PluginEventHandler = (String, EventHandler);

pub struct EventRegistry {
    registry: Mutex<HashMap<String, BTreeMap<i8, PluginEventHandler>>>,
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

    pub async fn register_handler(&self, plugin_name: String, handler: EventHandler) {
        self.registry
            .lock()
            .await
            .entry(handler.event_name.to_string())
            .or_default()
            .insert(handler.priority, (plugin_name, handler));
    }

    pub async fn call_event<T: Event>(&self, manager: &mut PluginManager, mut event: T) -> T {
        let lock = self.registry.lock().await;
        let Some(handlers) = lock.get(T::NAME) else {
            return event;
        };
        let mut cancelled = false;
        for (plugin_name, handler) in handlers.values() {
            if cancelled && !handler.flags.contains(EventHandlerFlags::RECEIVE_CANCELLED) {
                continue;
            }

            let instance = manager.get_mut(plugin_name).unwrap();
            let result = instance
                .on_event(&handler.handler_name, &event)
                .await
                .unwrap();

            if let Some((ptr, len)) = result.unpack() {
                // TODO: index bounds?
                let (cancelled_data, data) = instance.memory.data(&mut instance.store)
                    [ptr as usize..(ptr + len) as usize]
                    .split_at(1);

                if !data.is_empty() {
                    event = rmp_serde::from_slice(data).unwrap();
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

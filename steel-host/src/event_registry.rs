use std::collections::{BTreeMap, HashMap};

use crate::PluginManager;
use steel_plugin_sdk::event::{Event, EventHandlerFlags, handler::EventHandler};
use tokio::sync::Mutex;
use tracing::{error, warn};

/// A plugin-provided handler paired with its plugin name.
type PluginEventHandler = (String, EventHandler);

/// Stores all registered events and their handlers.
pub struct EventRegistry {
    /// Maps event names to their handlers.
    registry: Mutex<HashMap<String, BTreeMap<i8, Vec<PluginEventHandler>>>>,
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

    /// Registers a new event in the registry.
    ///
    /// # Arguments
    /// * `event_name` - Unique name of the event to register.
    pub async fn register_event(&self, event_name: String) {
        let mut lock = self.registry.lock().await;
        if lock.contains_key(&event_name) {
            warn!("Event '{}' is already registered.", event_name);
            return;
        }
        lock.insert(event_name, BTreeMap::new());
    }

    /// Registers a handler for an existing event.
    ///
    /// # Arguments
    /// * `plugin_name` - Name of the plugin providing the handler.
    /// * `handler` - Metadata describing the handler and the target event.
    pub async fn register_handler(&self, plugin_name: String, handler: EventHandler) {
        if let Some(handler_map) = self.registry.lock().await.get_mut(&*handler.event_name) {
            handler_map
                .entry(handler.priority)
                .or_default()
                .push((plugin_name, handler));
        } else {
            error!(
                "Plugin '{}' attempted to register a handler for unknown event '{}'",
                plugin_name, handler.event_name,
            );
        }
    }

    /// Dispatches an event to all registered handlers for its type.
    ///
    /// Handlers are executed in order of their priority. If an event
    /// has been canceled, handlers without the `RECEIVE_CANCELLED`
    /// flag are skipped.
    ///
    /// # Arguments
    /// * `manager` - The plugin manager used to look up plugin instances.
    /// * `event` - The event to dispatch.
    pub async fn call_event<T: Event>(&self, manager: &mut PluginManager, event: &mut T) {
        let lock = self.registry.lock().await;
        let Some(handler_map) = lock.get(T::NAME) else {
            return;
        };
        for handler_list in handler_map.values() {
            for (plugin_name, handler) in handler_list {
                if event.cancelled()
                    && !handler.flags.contains(EventHandlerFlags::RECEIVE_CANCELLED)
                {
                    continue;
                }

                let instance = manager.get_mut(plugin_name).unwrap();
                let result = instance
                    .on_event(&handler.handler_name, event)
                    .await
                    .unwrap();

                if let Some((ptr, len)) = result.unpack() {
                    let data = &instance.memory.data(&mut instance.store)
                        [ptr as usize..(ptr + len) as usize];
                    *event = rmp_serde::from_slice(data).unwrap();
                    instance.dealloc(ptr, len).await.unwrap();
                }
            }
        }
    }
}

use std::collections::{BTreeMap, HashMap};

use crate::{PluginManager, utils::memory::PluginMemory};
use parking_lot::Mutex;
use steel_plugin_sdk::event::{Event, EventHandlerFlags, handler::EventHandler};
use tracing::{error, warn};

/// A plugin-provided handler paired with its plugin name.
type PluginEventHandler = (String, EventHandler);

/// Stores all registered events and their handlers.
pub struct EventRegistry {
    /// Maps event names to their handlers.
    handlers: Mutex<HashMap<String, BTreeMap<i8, Vec<PluginEventHandler>>>>,
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
            handlers: Mutex::new(HashMap::new()),
        }
    }

    /// Registers a new event in the registry.
    ///
    /// # Arguments
    /// * `event_name` - Unique name of the event to register.
    pub fn register_event(&self, event_name: String) {
        let mut lock = self.handlers.lock();
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
    pub fn register_handler(&self, plugin_name: String, handler: EventHandler) {
        if let Some(handler_map) = self.handlers.lock().get_mut(&*handler.event_name) {
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

    /// Unregister all handlers belonging to a specific plugin.
    ///
    /// # Arguments
    /// * `plugin_name` - Name of the plugin providing the handler.
    pub fn unregister_handlers(&self, plugin_name: &str) {
        let mut handlers = self.handlers.lock();
        for handlers_list in handlers.values_mut() {
            for handler in handlers_list.values_mut() {
                handler.retain(|(name, _)| name != plugin_name);
            }
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
    pub async fn dispatch<T: Event>(&self, manager: &mut PluginManager, event: &mut T) {
        let Some(handler_map) = self.handlers.lock().get(T::NAME).cloned() else {
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

                let memory = PluginMemory::new(instance.memory, &mut instance.store);
                if let Some(fat) = result.fat {
                    *event = memory.read_msgpack(fat);
                    instance.dealloc(fat).await.unwrap();
                }
            }
        }
    }
}

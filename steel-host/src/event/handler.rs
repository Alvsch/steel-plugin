use crate::{event::dispatch_event, plugin::PluginStore};
use std::collections::HashMap;
use steel_plugin_sdk::event::TopicId;
use tracing::error;
use wasmtime::TypedFunc;

pub type HandlerFn = TypedFunc<u64, u64>;

pub struct HandlerEntry {
    pub store: PluginStore,
    pub handler_fn: HandlerFn,
    pub priority: i8,
}

pub struct HandlerRegistry {
    handlers: HashMap<TopicId, Vec<HandlerEntry>>,
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HandlerRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn subscribe(
        &mut self,
        topic_id: TopicId,
        plugin_store: PluginStore,
        handler_fn: HandlerFn,
        priority: i8,
    ) {
        let entries = self.handlers.entry(topic_id).or_default();
        let entry = HandlerEntry {
            store: plugin_store,
            handler_fn,
            priority,
        };
        let pos = entries.partition_point(|e| e.priority <= priority);
        entries.insert(pos, entry);
    }

    pub async fn dispatch_topic(&self, topic_id: TopicId, payload: &mut Vec<u8>) {
        let handlers = self.get_handlers(topic_id);
        for handler in handlers {
            let mut store = handler.store.lock().await;
            if let Err(err) = dispatch_event(&mut store, payload, &handler.handler_fn).await {
                error!("plugin contract violation during event dispatch: {err}");
            }
        }
    }

    #[must_use]
    fn get_handlers(&self, topic_id: TopicId) -> &[HandlerEntry] {
        self.handlers
            .get(&topic_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

use crate::{
    linker::event::{WasmEvent, dispatch_event},
    plugin::PluginInstance,
};
use std::collections::HashMap;
use steel_plugin_core::TopicId;
use tracing::error;

pub type HandlerFn = u32;

struct HandlerEntry {
    pub plugin: PluginInstance,
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
        plugin_store: PluginInstance,
        handler_fn: HandlerFn,
        priority: i8,
    ) {
        let entries = self.handlers.entry(topic_id).or_default();
        let entry = HandlerEntry {
            plugin: plugin_store,
            handler_fn,
            priority,
        };
        let pos = entries.partition_point(|e| e.priority <= priority);
        entries.insert(pos, entry);
    }

    pub async fn dispatch_topic<E: WasmEvent + Clone>(&self, event: E) -> wasmtime::Result<()> {
        let handlers = self.get_handlers(event.topic_id());
        for handler in handlers {
            if let Err(err) =
                dispatch_event(&handler.plugin, event.clone(), handler.handler_fn).await
            {
                error!("plugin contract violation during event dispatch: {err}");
            }
        }
        Ok(())
    }

    #[must_use]
    fn get_handlers(&self, topic_id: TopicId) -> &[HandlerEntry] {
        self.handlers
            .get(&topic_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

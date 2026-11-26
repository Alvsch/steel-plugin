use crate::{
    linker::event::{WasmEvent, dispatch_event},
    state::HostState,
};
use std::collections::HashMap;
use steel_plugin_core::TopicId;
use tracing::error;

pub(crate) type HandlerFn = u32;

struct HandlerEntry {
    pub plugin_name: String,
    pub handler_fn: HandlerFn,
    pub priority: i8,
}

pub(crate) struct HandlerRegistry {
    handlers: HashMap<TopicId, Vec<HandlerEntry>>,
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HandlerRegistry {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub(crate) fn subscribe(
        &mut self,
        topic_id: TopicId,
        plugin_name: String,
        handler_fn: HandlerFn,
        priority: i8,
    ) {
        let entries = self.handlers.entry(topic_id).or_default();
        let entry = HandlerEntry {
            plugin_name,
            handler_fn,
            priority,
        };
        let pos = entries.partition_point(|e| e.priority <= priority);
        entries.insert(pos, entry);
    }

    pub(crate) async fn dispatch_topic<E: WasmEvent + Clone>(
        &self,
        event: E,
        host: &HostState,
    ) -> wasmtime::Result<()> {
        let handlers = self.get_handlers(event.topic_id());
        for handler in handlers {
            let Some(plugin) = host.resolve_plugin(&handler.plugin_name) else {
                continue;
            };
            if let Err(err) = dispatch_event(&plugin, event.clone(), handler.handler_fn).await {
                error!("plugin contract violation during event dispatch: {err}");
            }
        }
        Ok(())
    }

    pub(crate) fn unsubscribe_plugin(&mut self, plugin_name: &str) {
        self.handlers.retain(|_, handlers| {
            handlers.retain(|handler| handler.plugin_name != plugin_name);
            !handlers.is_empty()
        });
    }

    #[must_use]
    fn get_handlers(&self, topic_id: TopicId) -> &[HandlerEntry] {
        self.handlers.get(&topic_id).map_or_default(Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::{HandlerFn, HandlerRegistry};
    use steel_plugin_core::TopicId;

    #[test]
    fn unsubscribe_removes_all_handlers_for_a_plugin() {
        let mut registry = HandlerRegistry::new();
        let topic = TopicId(7);
        let handler: HandlerFn = 1;

        registry.subscribe(topic, "plugin".to_owned(), handler, 0);
        assert_eq!(registry.get_handlers(topic).len(), 1);

        registry.unsubscribe_plugin("plugin");
        assert!(registry.get_handlers(topic).is_empty());
    }
}

use crate::plugin::PluginStore;
use std::collections::HashMap;
use steel_plugin_sdk::event::TopicId;

pub struct HandlerEntry {
    pub store: PluginStore,
    pub fn_table_index: u32,
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
        fn_table_index: u32,
        priority: i8,
    ) {
        let entries = self.handlers.entry(topic_id).or_default();
        let entry = HandlerEntry {
            store: plugin_store,
            fn_table_index,
            priority,
        };
        let pos = entries.partition_point(|e| e.priority <= priority);
        entries.insert(pos, entry);
    }

    #[must_use]
    pub fn get_handlers(&self, topic_id: TopicId) -> &[HandlerEntry] {
        self.handlers
            .get(&topic_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

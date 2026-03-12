use std::collections::HashMap;
use steel_plugin_sdk::event::TopicId;

pub struct TopicRegistry {
    topics: HashMap<String, TopicId>,
    next_id: u32,
}

impl Default for TopicRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TopicRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn register_topic(&mut self, name: impl Into<String>) {
        let topic_id = self.next_id;
        self.next_id += 1;
        self.topics.insert(name.into(), topic_id);
    }
}

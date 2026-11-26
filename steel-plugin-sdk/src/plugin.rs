use std::sync::OnceLock;

use crate::{
    event::Event,
    registration::{EventHandler, RpcMethod},
    sdk, warn, wit,
};

fn event_handlers() -> &'static [&'static EventHandler] {
    static HANDLERS: OnceLock<Vec<&'static EventHandler>> = OnceLock::new();
    HANDLERS.get_or_init(|| inventory::iter::<EventHandler>().collect())
}

/// Lifecycle hooks implemented by a plugin type.
pub trait Plugin {
    fn on_enable();
    fn on_disable();
}

impl<T: Plugin> wit::Guest for T {
    fn on_enable() {
        // Keep registration and dispatch IDs tied to the same cached ordering.
        for (index, handler) in event_handlers().iter().enumerate() {
            let Ok(handler_id) = u32::try_from(index) else {
                warn!("plugin has too many event handlers to assign IDs");
                break;
            };
            sdk::event::register_event(handler_id, handler.topic_id.0, i32::from(handler.priority));
        }

        T::on_enable();
    }

    fn on_disable() {
        T::on_disable();
    }

    fn rpc(method_name: String, data: Vec<u8>) -> Result<Option<Vec<u8>>, sdk::rpc::RpcError> {
        for method in inventory::iter::<RpcMethod> {
            if method.name == method_name {
                return Ok((method.function)(&data));
            }
        }
        Err(sdk::rpc::RpcError::UnknownMethod)
    }

    fn event_handler(handler_id: u32, mut event: Event) {
        let Ok(index) = usize::try_from(handler_id) else {
            warn!("host called for unknown handler id: {handler_id}");
            return;
        };
        let Some(handler) = event_handlers().get(index) else {
            warn!("host called for unknown handler id: {handler_id}");
            return;
        };
        (handler.function)(&mut event);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use steel_plugin_core::TopicId;

    use super::{Event, EventHandler, Plugin, event_handlers, wit};

    static FIRST_HANDLER_CALLS: AtomicUsize = AtomicUsize::new(0);
    static SECOND_HANDLER_CALLS: AtomicUsize = AtomicUsize::new(0);

    fn first_handler(_: &mut Event) {
        FIRST_HANDLER_CALLS.fetch_add(1, Ordering::Relaxed);
    }

    fn second_handler(_: &mut Event) {
        SECOND_HANDLER_CALLS.fetch_add(1, Ordering::Relaxed);
    }

    inventory::submit! {
        EventHandler {
            topic_id: TopicId(1),
            priority: 0,
            function: first_handler,
        }
    }

    inventory::submit! {
        EventHandler {
            topic_id: TopicId(1),
            priority: 0,
            function: second_handler,
        }
    }

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn on_enable() {}

        fn on_disable() {}
    }

    #[test]
    fn event_handler_ids_select_distinct_handlers() {
        assert_eq!(event_handlers().len(), 2);

        <TestPlugin as wit::Guest>::event_handler(0, Event::Ping);
        <TestPlugin as wit::Guest>::event_handler(1, Event::Ping);

        assert_eq!(FIRST_HANDLER_CALLS.load(Ordering::Relaxed), 1);
        assert_eq!(SECOND_HANDLER_CALLS.load(Ordering::Relaxed), 1);
    }
}

use crate::{
    event::EventHandler,
    rpc::export::RpcMethod,
    sdk::{self, event::Event},
    warn, wit,
};

pub trait Plugin {
    fn on_enable();
    fn on_disable();
}

impl<T: Plugin> wit::Guest for T {
    fn on_enable() {
        for handler in inventory::iter::<EventHandler> {
            sdk::event::register_event(handler.id, handler.topic_id.0, i32::from(handler.priority));
        }

        T::on_enable();
    }

    fn on_disable() {
        T::on_disable();
    }

    fn rpc(method_name: String, data: Vec<u8>) -> Option<Vec<u8>> {
        for method in inventory::iter::<RpcMethod> {
            if method.name == method_name {
                return (method.function)(&data);
            }
        }
        None
    }

    fn event_handler(handler_id: u32, mut event: Event) {
        for handler in inventory::iter::<EventHandler> {
            if handler.id == handler_id {
                (handler.function)(&mut event);
                return;
            }
        }
        warn!("host called for unknown handler id: {handler_id}");
    }
}

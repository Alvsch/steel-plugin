use crate::{
    plugin::component::exports::host::plugin_sdk::plugin_api::Guest, rpc::export::RpcMethod,
};

#[doc(hidden)]
#[allow(clippy::all, clippy::pedantic)]
pub mod component {
    wit_bindgen::generate!({
        path: "../wit",
        world: "plugin-world",
    });
}

pub trait Plugin {
    fn on_enable();
    fn on_disable();
}

impl<T: Plugin> Guest for T {
    fn on_enable() {
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

    fn event_handler(_handler_id: u32, _data: Vec<u8>) {}
}

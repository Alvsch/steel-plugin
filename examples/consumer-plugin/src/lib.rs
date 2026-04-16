use steel_plugin_sdk::{Guest, export, rpc};
use steel_plugin_sdk::info;

pub struct ConsumerPlugin;

impl Guest for ConsumerPlugin {
    fn on_enable() {
        info!("hello from the consumer!");

        let plugin_id = rpc::resolve_plugin("provider-plugin").unwrap();
        let method_id = rpc::resolve_method(plugin_id, "get_balance").unwrap();
        let result =
            rpc::dispatch(plugin_id, method_id, b"hello").and_then(|x| String::from_utf8(x).ok());

        info!("{result:?}");
    }

    fn on_disable() {}

    fn on_load() -> Vec<u8> {
        Vec::new()
    }

    fn rpc(method_id: u32, data: Vec<u8>) -> Option<Vec<u8>> {
        info!("rpc");
        None
    }

    fn event_handler(handler_id: u32, data: Vec<u8>) {
        info!("event");
    }
}

// #[on_enable]
// pub fn on_enable() {
//     info!("hello from the consumer!");

//     let plugin_id = rpc::resolve_plugin("provider-plugin").unwrap();
//     let method_id = rpc::resolve_method(plugin_id, "get_balance").unwrap();
//     let result =
//         rpc::dispatch(plugin_id, method_id, b"hello").and_then(|x| String::from_utf8(x).ok());

//     info!("{result:?}");
// }

export!(ConsumerPlugin);

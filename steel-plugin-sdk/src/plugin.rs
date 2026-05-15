use crate::{
    component::exports::host::plugin_sdk::plugin_api::Guest,
    export::{Exported, ExportedId},
};

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

    fn on_load() -> Vec<u8> {
        let slice = inventory::iter::<Exported>()
            .cloned()
            .map(ExportedId::from)
            .collect::<Vec<_>>();
        rmp_serde::to_vec(&slice).expect("invalid exports")
    }

    fn rpc(_method_id: u32, _data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }

    fn event_handler(_handler_id: u32, _data: Vec<u8>) {}
}

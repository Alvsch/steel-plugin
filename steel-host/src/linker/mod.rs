use wasmtime::component::{HasSelf, Linker};

use crate::{linker::host::plugin_sdk, plugin::PluginState};

pub mod event;
pub mod logging;
pub mod player;
pub mod rpc;

pub type HostLinker = Linker<PluginState>;

wasmtime::component::bindgen!({
    path: "../wit",
    exports: { default: async | trappable },
    imports: { default: async },
});

pub fn add_to_linker(linker: &mut HostLinker) -> wasmtime::Result<()> {
    plugin_sdk::logging::add_to_linker::<_, HasSelf<_>>(linker, |state| state)?;
    plugin_sdk::player::add_to_linker::<_, HasSelf<_>>(linker, |state| state)?;
    plugin_sdk::rpc::add_to_linker::<_, HasSelf<_>>(linker, |state| state)?;
    plugin_sdk::event::add_to_linker::<_, HasSelf<_>>(linker, |state| state)?;
    Ok(())
}

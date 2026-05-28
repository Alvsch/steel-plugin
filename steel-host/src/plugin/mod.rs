use std::sync::Arc;

use steel_utils::locks::AsyncMutex;
use wasmtime::Store;

use crate::linker::PluginWorld;
pub use crate::plugin::{
    resource::PluginResources,
    state::{PluginState, PluginStatus},
};

mod resource;
mod state;

pub type PluginInstance = Arc<Plugin>;
pub type PluginStore = Store<PluginState>;

pub struct Plugin {
    pub store: AsyncMutex<PluginStore>,
    pub bindings: AsyncMutex<PluginWorld>,
}

use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::RwLock;

use crate::{plugin::PluginStore, rpc::host::HostRpc};

pub struct HostState {
    pub rpc: ArcSwap<HostRpc>,
    pub enabled_plugins: RwLock<Vec<PluginStore>>,
    next_id: AtomicU32,
}

impl Default for HostState {
    fn default() -> Self {
        Self::new()
    }
}

impl HostState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rpc: ArcSwap::from_pointee(HostRpc::new()),
            enabled_plugins: RwLock::new(Vec::new()),
            next_id: AtomicU32::new(0),
        }
    }

    pub fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

use crate::plugin::PluginStore;
use crate::rpc::HostRpc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::RwLock;

pub struct HostState {
    pub rpc: RwLock<HostRpc>,
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
            rpc: RwLock::new(HostRpc::new()),
            enabled_plugins: RwLock::new(Vec::new()),
            next_id: AtomicU32::new(0),
        }
    }

    pub fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

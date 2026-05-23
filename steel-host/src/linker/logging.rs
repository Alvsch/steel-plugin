use crate::{linker::host::plugin_sdk::logging, plugin::PluginState};

impl logging::Host for PluginState {
    async fn error(&mut self, message: String) {
        tracing::error!("[{}] {message}", self.meta.name);
    }

    async fn warn(&mut self, message: String) {
        tracing::warn!("[{}] {message}", self.meta.name);
    }

    async fn info(&mut self, message: String) {
        tracing::info!("[{}] {message}", self.meta.name);
    }

    async fn debug(&mut self, message: String) {
        tracing::debug!("[{}] {message}", self.meta.name);
    }

    async fn trace(&mut self, message: String) {
        tracing::trace!("[{}] {message}", self.meta.name);
    }
}

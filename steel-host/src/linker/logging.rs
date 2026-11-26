use crate::{linker::host::plugin_sdk::logging, plugin::PluginState};

impl logging::Host for PluginState {
    async fn error(&mut self, message: String) {
        tracing::error!(target: "steel_host", "[{}] {message}", self.meta.name);
    }

    async fn warn(&mut self, message: String) {
        tracing::warn!(target: "steel_host", "[{}] {message}", self.meta.name);
    }

    async fn info(&mut self, message: String) {
        tracing::info!(target: "steel_host", "[{}] {message}", self.meta.name);
    }

    async fn debug(&mut self, message: String) {
        tracing::debug!(target: "steel_host", "[{}] {message}", self.meta.name);
    }

    async fn trace(&mut self, message: String) {
        tracing::trace!(target: "steel_host", "[{}] {message}", self.meta.name);
    }
}

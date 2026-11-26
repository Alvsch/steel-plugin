//! Rust SDK for writing Steel plugins.
//!
//! The root exports and the [`event`] and [`rpc`] modules form the plugin-facing
//! API. [`sdk`] exposes the generated WIT bindings for callers that need direct
//! access to the component interface; prefer the SDK modules where possible.
//!
//! [`serde`] is re-exported for declaring typed RPC request and response values.

pub mod event {
    //! Typed event payloads exposed to plugin event handlers.
    pub use crate::sdk::event::{Event, PlayerJoinEvent, PlayerLeaveEvent};
}
mod logging;
mod plugin;
mod registration;
pub mod rpc;
pub use serde;

#[allow(clippy::all, clippy::pedantic)]
mod bindings;

/// Internal support items referenced by SDK procedural macro expansions and
/// generated WIT bindings. This is not a plugin-author API.
#[doc(hidden)]
pub mod __export {
    // Used by plugin_export!.
    pub use crate::bindings::{export, exports};

    // Used by #[event_handler] and #[rpc_export].
    pub use inventory::submit;

    // Used by #[event_handler].
    pub use crate::{event::Event, registration::EventHandler};
    pub use steel_plugin_core::TopicId;

    // Used by #[rpc_export].
    pub use crate::registration::RpcMethod;
    pub use crate::rpc::{decode_rpc_request, encode_rpc_response};

    // Used by error!, warn!, info!, debug!, and trace!.
    pub use crate::sdk::logging::{debug, error, info, trace, warn};
}

// Alias the generated guest interface used to implement the plugin component.
pub(crate) use crate::bindings::exports::host::plugin_sdk::plugin_api as wit;

/// Generated WIT bindings for the plugin world.
///
/// This low-level module is exposed for compatibility and advanced use. Prefer
/// the SDK's curated modules when writing plugins.
pub use crate::bindings::host::plugin_sdk as sdk;
pub use plugin::Plugin;
pub use steel_plugin_macros::{event_handler, plugin_export, rpc_export};

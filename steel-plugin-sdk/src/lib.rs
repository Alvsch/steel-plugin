mod event;
mod plugin;
pub mod rpc;

#[doc(hidden)]
#[allow(clippy::all, clippy::pedantic)]
mod bindings;

#[doc(hidden)]
pub mod __export {
    pub use inventory::submit;

    pub use crate::{
        bindings::{export, exports},
        event::EventHandler,
        sdk::{
            event::Event,
            logging::{debug, error, info, trace, warn},
        },
    };
    pub use steel_plugin_core::TopicId;
}

pub(crate) use crate::bindings::exports::host::plugin_sdk::plugin_api as wit;
pub use crate::bindings::host::plugin_sdk as sdk;
pub use plugin::Plugin;
pub use steel_plugin_macros::{event_handler, plugin_export, rpc_export};

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::__export::error(&message);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::__export::warn(&message);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::__export::info(&message);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::__export::debug(&message);
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::__export::trace(&message);
    };
}

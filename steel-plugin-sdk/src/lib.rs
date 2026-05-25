pub use steel_plugin_macros::{event_handler, plugin_export, rpc_export};

pub mod event;
mod plugin;
pub mod rpc;

pub use plugin::Plugin;
pub use steel_plugin_core::{STEEL_API_VERSION, TopicId};

#[doc(hidden)]
pub mod __export {
    pub use inventory::submit;

    pub use crate::sdk::logging::debug;
    pub use crate::sdk::logging::error;
    pub use crate::sdk::logging::info;
    pub use crate::sdk::logging::trace;
    pub use crate::sdk::logging::warn;

    pub use crate::plugin::component::exports::host::plugin_sdk;
    pub use crate::plugin::component::host::plugin_sdk::event::Event;
}

pub(crate) use crate::plugin::component::host::plugin_sdk as sdk;

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

pub use steel_plugin_macros::{event_handler, plugin_export, rpc_export};

pub mod event;
pub mod export;
pub mod objects;
mod plugin;
pub mod rpc;

pub use plugin::Plugin;
pub use steel_plugin_core::STEEL_API_VERSION;

#[doc(hidden)]
pub mod __export {
    pub use crate::sdk::logging::debug;
    pub use crate::sdk::logging::error;
    pub use crate::sdk::logging::info;
    pub use crate::sdk::logging::trace;
    pub use crate::sdk::logging::warn;
}

#[doc(hidden)]
#[allow(clippy::all, clippy::pedantic)]
pub mod component {
    wit_bindgen::generate!({
        path: "../wit",
        world: "plugin-world",
    });
}
pub(crate) use component::host::plugin_sdk as sdk;

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        unsafe {
            $crate::__export::error(&message);
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        unsafe {
            $crate::__export::warn(&message);
        }
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        unsafe {
            $crate::__export::info(&message);
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        unsafe {
            $crate::__export::debug(&message);
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        unsafe {
            $crate::__export::trace(&message);
        }
    };
}

pub use steel_plugin_macros::{event_handler, on_disable, on_enable, plugin_meta, rpc_export};

pub mod event;
pub mod export;
pub mod objects;
pub mod rpc;

pub use steel_plugin_core::STEEL_API_VERSION;

#[allow(clippy::all, clippy::pedantic)]
mod host {
    wit_bindgen::generate!({
        path: "../wit",
        world: "plugin-world",
    });
}

pub use crate::host::{exports::host::plugin_sdk::plugin_api::Guest};
pub(crate) use host::host::plugin_sdk as sdk;

#[doc(hidden)]
pub mod __export {
    pub use crate::sdk::logging::debug;
    pub use crate::sdk::logging::error;
    pub use crate::sdk::logging::info;
    pub use crate::sdk::logging::trace;
    pub use crate::sdk::logging::warn;
}

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

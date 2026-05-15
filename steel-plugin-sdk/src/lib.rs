pub use steel_plugin_macros::{event_handler, on_disable, on_enable, plugin_meta, rpc_export};

#[allow(clippy::all, clippy::pedantic)]
pub mod component {
    wit_bindgen::generate!({
        path: "../wit",
        world: "plugin-world",
    });
}
pub mod event;
pub mod export;
pub mod objects;
mod plugin;
pub mod rpc;

pub(crate) use component::host::plugin_sdk as sdk;
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

#[macro_export]
macro_rules! plugin_export {
    ($plugin:ty) => {
        const _: () = {
            #[unsafe(export_name = "host:plugin-sdk/plugin-api@0.1.0#on-enable")]
            unsafe extern "C" fn export_on_enable() {
                unsafe {
                    $crate::component::exports::host::plugin_sdk::plugin_api::_export_on_enable_cabi::<$plugin>()
                }
            }

            #[unsafe(export_name = "host:plugin-sdk/plugin-api@0.1.0#on-disable")]
            unsafe extern "C" fn export_on_disable() {
                unsafe {
                    $crate::component::exports::host::plugin_sdk::plugin_api::_export_on_disable_cabi::<$plugin>()
                }
            }

            #[unsafe(export_name = "host:plugin-sdk/plugin-api@0.1.0#on-load")]
            unsafe extern "C" fn export_on_load() -> *mut u8 {
                unsafe {
                    $crate::component::exports::host::plugin_sdk::plugin_api::_export_on_load_cabi::<$plugin>()
                }
            }

            #[unsafe(export_name = "cabi_post_host:plugin-sdk/plugin-api@0.1.0#on-load")]
            unsafe extern "C" fn _post_return_on_load(arg0: *mut u8) {
                unsafe {
                    $crate::component::exports::host::plugin_sdk::plugin_api::__post_return_on_load::<$plugin>(arg0)
                }
            }

            #[unsafe(export_name = "host:plugin-sdk/plugin-api@0.1.0#rpc")]
            unsafe extern "C" fn export_rpc(arg0: i32, arg1: *mut u8, arg2: usize) -> *mut u8 {
                unsafe {
                    $crate::component::exports::host::plugin_sdk::plugin_api::_export_rpc_cabi::<$plugin>(arg0, arg1, arg2)
                }
            }

            #[unsafe(export_name = "cabi_post_host:plugin-sdk/plugin-api@0.1.0#rpc")]
            unsafe extern "C" fn _post_return_rpc(arg0: *mut u8) {
                unsafe {
                    $crate::component::exports::host::plugin_sdk::plugin_api::__post_return_rpc::<$plugin>(arg0)
                }
            }

            #[unsafe(export_name = "host:plugin-sdk/plugin-api@0.1.0#event-handler")]
            unsafe extern "C" fn export_event_handler(arg0: i32, arg1: *mut u8, arg2: usize) {
                unsafe {
                    $crate::component::exports::host::plugin_sdk::plugin_api::_export_event_handler_cabi::<$plugin>(arg0, arg1, arg2)
                }
            }
        };
    };
}

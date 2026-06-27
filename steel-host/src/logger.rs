use mlua::UserData;
use mlua::prelude::LuaUserDataMethods;
use tracing::{debug, error, info, trace, warn};

pub struct Logger;

impl UserData for Logger {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("Trace", |_, message: String| {
            trace!("{message}");
            Ok(())
        });
        methods.add_function("Debug", |_, message: String| {
            debug!("{message}");
            Ok(())
        });
        methods.add_function("Info", |_, message: String| {
            info!("{message}");
            Ok(())
        });
        methods.add_function("Warn", |_, message: String| {
            warn!("{message}");
            Ok(())
        });
        methods.add_function("Error", |_, message: String| {
            error!("{message}");
            Ok(())
        });
    }
}

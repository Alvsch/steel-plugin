use std::sync::Weak;

use mlua::prelude::*;
use steel_utils::locks::SyncMutex;

use crate::api::signal::CallbackMap;

slotmap::new_key_type! {
    pub(crate) struct ConnectionKey;
}

pub struct Connection<T> {
    pub(crate) key: ConnectionKey,
    pub(crate) callbacks: Weak<SyncMutex<CallbackMap<T>>>,
}

impl<T> Connection<T> {
    pub fn disconnect(self) {
        if let Some(callbacks) = self.callbacks.upgrade() {
            callbacks.lock().remove(self.key);
        }
    }
}

impl<T> LuaUserData for Connection<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_once("Disconnect", |_, this, ()| {
            this.disconnect();
            Ok(())
        });
    }
}
#[cfg(test)]
mod tests {
    use mlua::prelude::*;

    use crate::api::Signal;

    #[test]
    fn connection_disconnect() {
        let lua = Lua::new();
        let globals = lua.globals();

        let signal: Signal<String> = Signal::new();
        globals
            .set("signal", signal.clone())
            .expect("failed to set global");

        lua.load(
            "
            times = 0
            connection = signal:Connect(function(v: string)
                times += 1
            end)
        ",
        )
        .exec()
        .expect("failed to execute Connect script");

        signal.emit("test".to_string());
        signal.emit("test".to_string());

        assert!(globals.get::<u32>("times").is_ok_and(|x| x == 2));

        lua.load(
            r"
            connection:Disconnect()
        ",
        )
        .exec()
        .expect("failed to disconnect connection");

        signal.emit("test".to_string());

        assert!(globals.get::<u32>("times").is_ok_and(|x| x == 2));
    }
}

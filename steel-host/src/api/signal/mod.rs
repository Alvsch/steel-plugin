use std::{marker::PhantomData, sync::Arc};

use mlua::{Error, prelude::*};
use slotmap::SlotMap;
use steel_utils::locks::SyncMutex;
use tokio::sync::broadcast::{Sender, channel, error::RecvError};

use crate::api::signal::connection::ConnectionKey;

pub use connection::Connection;
mod connection;

pub(crate) type CallbackMap<T> = SlotMap<ConnectionKey, CallbackEntry<T>>;

pub(crate) struct CallbackEntry<T> {
    callback: LuaFunction,
    once: bool,
    _marker: PhantomData<fn(T)>,
}

#[derive(Clone)]
pub struct Signal<T: IntoLua + Clone + 'static> {
    callbacks: Arc<SyncMutex<CallbackMap<T>>>,
    sender: Sender<T>,
    userland: bool,
    _marker: PhantomData<fn(T)>,
}

impl<T: IntoLua + Clone + 'static> Default for Signal<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: IntoLua + Clone> Signal<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(SyncMutex::new(SlotMap::with_key())),
            sender: channel(16).0,
            userland: false,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn new_userland() -> Self {
        Self {
            callbacks: Arc::new(SyncMutex::new(SlotMap::with_key())),
            sender: channel(16).0,
            userland: true,
            _marker: PhantomData,
        }
    }

    pub fn connect(&self, callback: LuaFunction, once: bool) -> Connection<T> {
        let key = self.callbacks.lock().insert(CallbackEntry {
            callback,
            once,
            _marker: PhantomData,
        });
        Connection {
            key,
            callbacks: Arc::downgrade(&self.callbacks),
        }
    }

    pub fn disconnect_all(&self) {
        self.callbacks.lock().clear();
    }

    pub async fn wait(&self) -> LuaResult<T> {
        let mut receiver = self.sender.subscribe();
        loop {
            match receiver.recv().await {
                Ok(value) => return Ok(value),
                Err(RecvError::Lagged(_)) => (),
                Err(error) => return Err(Error::external(error)),
            }
        }
    }

    pub fn emit(&self, value: T) {
        let to_call = {
            let mut lock = self.callbacks.lock();

            let mut to_call = Vec::with_capacity(lock.len());
            lock.retain(|_, entry| {
                to_call.push(entry.callback.clone());
                !entry.once
            });
            to_call
        };

        for callback in to_call {
            callback.call::<()>(value.clone()).expect("error");
        }

        self.sender.send(value).ok();
    }
}

impl<T> LuaUserData for Signal<T>
where
    T: IntoLua + FromLuaMulti + Clone + Send + Sync + 'static,
{
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |_, ()| Ok(Signal::<T>::new_userland()));
        methods.add_method("Connect", |_, this, cb: LuaFunction| {
            let connection = this.connect(cb, false);
            Ok(connection)
        });
        methods.add_method("Once", |_, this, cb: LuaFunction| {
            let connection = this.connect(cb, true);
            Ok(connection)
        });
        methods.add_async_method("Wait", async |_, this, ()| this.wait().await);
        methods.add_method("Emit", |_, this, value: T| {
            if this.userland {
                this.emit(value);
            }
            Ok(())
        });
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use mlua::prelude::*;
    use tokio::task::yield_now;
    use tokio::time::timeout;

    use crate::api::Signal;

    #[test]
    fn signal_connect() {
        let lua = Lua::new();
        let globals = lua.globals();

        let signal: Signal<String> = Signal::new();
        globals
            .set("signal", signal.clone())
            .expect("failed to set global");

        lua.load(
            r#"
            times = 0
            signal:Connect(function(v: string)
                assert(v == "test")
                times += 1
            end)
        "#,
        )
        .exec()
        .expect("failed to register Connect callback");

        signal.emit("test".to_string());
        signal.emit("test".to_string());
        signal.emit("test".to_string());

        assert!(globals.get::<u32>("times").is_ok_and(|x| x == 3));
    }

    #[test]
    fn signal_once() {
        let lua = Lua::new();
        let globals = lua.globals();

        let signal: Signal<String> = Signal::new();
        globals
            .set("signal", signal.clone())
            .expect("failed to set global");

        lua.load(
            r#"
            times = 0
            signal:Once(function(v: string)
                assert(v == "test")
                times += 1
            end)
        "#,
        )
        .exec()
        .expect("failed to register Once callback");

        signal.emit("test".to_string());
        signal.emit("test".to_string());
        signal.emit("test".to_string());

        assert!(globals.get::<u32>("times").is_ok_and(|x| x == 1));
    }

    #[tokio::test]
    async fn signal_wait() {
        let lua = Lua::new();
        let globals = lua.globals();

        let signal: Signal<String> = Signal::new();
        globals
            .set("signal", signal.clone())
            .expect("failed to set global");

        let future = lua
            .load(
                r"
            result = signal:Wait()
        ",
            )
            .exec_async();
        let task = tokio::spawn(future);

        yield_now().await;

        let value = "test";
        signal.emit(value.to_string());

        timeout(Duration::from_secs(1), task)
            .await
            .expect("signal:Wait task timed out")
            .expect("signal:Wait task failed to join")
            .expect("signal:Wait task returned an error");

        assert!(globals.get::<String>("result").is_ok_and(|x| x == value));
    }

    #[tokio::test]
    async fn signal_wait_multiple() {
        let lua = Lua::new();
        let globals = lua.globals();

        let signal: Signal<String> = Signal::new();
        globals
            .set("signal", signal.clone())
            .expect("failed to set global");

        let first = lua
            .load(
                r"
            first = signal:Wait()
        ",
            )
            .exec_async();
        let second = lua
            .load(
                r"
            second = signal:Wait()
        ",
            )
            .exec_async();

        let first_task = tokio::spawn(first);
        let second_task = tokio::spawn(second);

        yield_now().await;

        signal.emit("test".to_string());

        timeout(Duration::from_secs(1), first_task)
            .await
            .expect("first Wait task timed out")
            .expect("first Wait task failed to join")
            .expect("first Wait task returned an error");
        timeout(Duration::from_secs(1), second_task)
            .await
            .expect("second Wait task timed out")
            .expect("second Wait task failed to join")
            .expect("second Wait task returned an error");

        assert!(globals.get::<String>("first").is_ok_and(|x| x == "test"));
        assert!(globals.get::<String>("second").is_ok_and(|x| x == "test"));
    }
}

use mlua::prelude::{LuaFunction, LuaUserDataFields, LuaUserDataMethods};
use mlua::{IntoLuaMulti, UserData};
use slab::Slab;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use steel_utils::locks::SyncMutex;
use tokio::sync::Notify;
use tracing::error;

#[derive(Debug)]
enum Callback {
    Persistent(LuaFunction),
    Once(LuaFunction),
}

type CallbackType = (Callback, Arc<AtomicBool>);

#[derive(Debug, Clone)]
pub struct Signal<T: Send + IntoLuaMulti + Clone + 'static> {
    callback: Arc<SyncMutex<Slab<CallbackType>>>,
    notify: Arc<Notify>,
    _marker: PhantomData<T>,
}

impl<T: Send + IntoLuaMulti + Clone> Default for Signal<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + IntoLuaMulti + Clone> Signal<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            callback: Arc::new(SyncMutex::new(Slab::new())),
            notify: Arc::new(Notify::new()),
            _marker: PhantomData,
        }
    }

    pub fn emit(&self, value: T) {
        self.callback.lock().retain(|_, (callback, alive)| {
            if !alive.load(Ordering::Acquire) {
                return false;
            }
            match callback {
                Callback::Persistent(func) => {
                    if let Err(e) = func.call::<()>(value.clone()) {
                        error!("Lua callback failed: {:?}", e);
                    }
                    true
                }
                Callback::Once(func) => {
                    alive.store(false, Ordering::Release);
                    if let Err(e) = func.call::<()>(value.clone()) {
                        error!("Lua callback failed: {:?}", e);
                    }
                    false
                }
            }
        });
        self.notify.notify_waiters();
    }
}

impl<T: Send + IntoLuaMulti + Clone> UserData for Signal<T> {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Connect", |_, this, callback: LuaFunction| {
            let alive = Arc::new(AtomicBool::new(true));
            this.callback
                .lock()
                .insert((Callback::Persistent(callback), alive.clone()));
            Ok(Connection { connected: alive })
        });
        methods.add_method("Once", |_, this, callback: LuaFunction| {
            let alive = Arc::new(AtomicBool::new(true));
            this.callback
                .lock()
                .insert((Callback::Once(callback), alive.clone()));
            Ok(Connection { connected: alive })
        });
        methods.add_async_method("Wait", async |_, this, ()| {
            this.notify.notified().await;
            Ok(())
        });
    }
}

pub struct Connection {
    connected: Arc<AtomicBool>,
}

impl UserData for Connection {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Connected", |_, this| {
            Ok(this.connected.load(Ordering::Acquire))
        });
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("Disconnect", |_, this, ()| {
            this.connected.store(false, Ordering::Release);
            Ok(())
        });
    }
}

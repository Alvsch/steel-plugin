use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};

use mlua::prelude::*;
use slotmap::SlotMap;
use steel_utils::locks::SyncMutex;
use tokio::sync::broadcast::{Sender, channel, error::RecvError};

slotmap::new_key_type! {
    struct ConnectionKey;
}

struct CallbackEntry<T> {
    callback: LuaFunction,
    once: bool,
    _marker: PhantomData<fn(T)>,
}

#[derive(Clone)]
pub struct Signal<T: IntoLua + Clone + 'static> {
    callbacks: Arc<SyncMutex<SlotMap<ConnectionKey, CallbackEntry<T>>>>,
    sender: Sender<T>,
    _marker: PhantomData<fn(T)>,
}

pub struct Connection<T> {
    key: ConnectionKey,
    callbacks: Weak<SyncMutex<SlotMap<ConnectionKey, CallbackEntry<T>>>>,
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
            sender: channel(1).0,
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

    pub async fn wait(&self) -> Result<T, RecvError> {
        self.sender.subscribe().recv().await
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

impl<T> Connection<T> {
    pub fn disconnect(self) {
        if let Some(callbacks) = self.callbacks.upgrade() {
            callbacks.lock().remove(self.key);
        }
    }
}

impl<T> LuaUserData for Signal<T>
where
    T: IntoLua + Clone + Send + Sync + 'static,
{
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Connect", |_, this, cb: LuaFunction| {
            this.connect(cb, false);
            Ok(())
        });
        methods.add_method("Once", |_, this, cb: LuaFunction| {
            this.connect(cb, true);
            Ok(())
        });
        methods.add_async_method("Wait", async |_, this, ()| {
            let value = this.wait().await.expect("error");
            Ok(value)
        });
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

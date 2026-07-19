use core::future::Future;

use mlua::prelude::*;

use crate::api::data_store::key::LuaKey;

pub use memory::MemoryStore;
mod key;
mod memory;

pub trait DataStore {
    fn set_async(&self, key: LuaKey, value: LuaValue)
    -> impl Future<Output = LuaResult<()>> + Send;
    fn update_async(
        &self,
        key: LuaKey,
        update: LuaFunction,
    ) -> impl Future<Output = LuaResult<()>> + Send;
    fn get_async(&self, key: LuaKey) -> impl Future<Output = LuaResult<LuaValue>> + Send;
    fn remove_async(&self, key: LuaKey) -> impl Future<Output = LuaResult<LuaValue>> + Send;
}

#[macro_export]
macro_rules! impl_store_userdata {
    ($ty:ty) => {
        impl mlua::prelude::LuaUserData for $ty
        where
            $ty: $crate::api::data_store::DataStore,
        {
            fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
                methods.add_async_method(
                    "SetAsync",
                    async |_, this, (key, value): (LuaKey, LuaValue)| {
                        this.set_async(key, value).await
                    },
                );
                methods.add_async_method(
                    "UpdateAsync",
                    async |_, this, (key, update): (LuaKey, LuaFunction)| {
                        this.update_async(key, update).await
                    },
                );
                methods.add_async_method("GetAsync", async |_, this, key: LuaKey| {
                    this.get_async(key).await
                });
                methods.add_async_method("RemoveAsync", async |_, this, key: LuaKey| {
                    this.remove_async(key).await
                });
            }
        }
    };
}

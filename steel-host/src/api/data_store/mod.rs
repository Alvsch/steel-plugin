use core::future::Future;

use mlua::prelude::*;

mod lmdb;
mod memory;

pub use lmdb::{LmdbStore, open_lmdb_env};
pub use memory::MemoryStore;

pub trait DataStore {
    fn set_async(&self, key: String, value: LuaValue)
    -> impl Future<Output = LuaResult<()>> + Send;
    fn get_async(&self, lua: &Lua, key: String)
    -> impl Future<Output = LuaResult<LuaValue>> + Send;
    fn update_async(
        &self,
        lua: &Lua,
        key: String,
        update: LuaFunction,
    ) -> impl Future<Output = LuaResult<()>> + Send;
    fn remove_async(
        &self,
        lua: &Lua,
        key: String,
    ) -> impl Future<Output = LuaResult<LuaValue>> + Send;
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
                    async |_, this, (key, value): (String, mlua::prelude::LuaValue)| {
                        this.set_async(key, value).await
                    },
                );
                methods.add_async_method(
                    "UpdateAsync",
                    async |lua, this, (key, update): (String, mlua::prelude::LuaFunction)| {
                        this.update_async(&lua, key, update).await
                    },
                );
                methods.add_async_method("GetAsync", async |lua, this, key: String| {
                    this.get_async(&lua, key).await
                });
                methods.add_async_method("RemoveAsync", async |lua, this, key: String| {
                    this.remove_async(&lua, key).await
                });
            }
        }
    };
}

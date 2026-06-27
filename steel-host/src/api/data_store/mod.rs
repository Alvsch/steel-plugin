use std::{collections::HashMap, sync::Arc};

use mlua::prelude::*;
use steel_utils::locks::AsyncMutex;

use crate::api::data_store::key::LuaKey;

mod key;

#[derive(Debug, Clone)]
pub struct DataStore {
    map: Arc<AsyncMutex<HashMap<LuaKey, LuaValue>>>,
}

impl Default for DataStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DataStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }
}

impl LuaUserData for DataStore {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method(
            "SetAsync",
            async |_, this, (key, value): (LuaKey, LuaValue)| {
                this.map.lock().await.insert(key, value);
                Ok(())
            },
        );
        methods.add_async_method(
            "UpdateAsync",
            async |_, this, (key, update): (LuaKey, LuaFunction)| {
                let mut map = this.map.lock().await;
                let current = map.get(&key).cloned();

                let updated = match update
                    .call_async::<LuaValue>(current.unwrap_or(LuaValue::Nil))
                    .await?
                {
                    LuaValue::Nil => None,
                    v => Some(v),
                };

                match updated {
                    Some(v) => {
                        map.insert(key, v);
                    }
                    None => {
                        map.remove(&key);
                    }
                }

                Ok(())
            },
        );
        methods.add_async_method("GetAsync", async |_, this, key: LuaKey| {
            Ok(this
                .map
                .lock()
                .await
                .get(&key)
                .cloned()
                .unwrap_or(LuaValue::Nil))
        });
        methods.add_async_method("RemoveAsync", async |_, this, key: LuaKey| {
            Ok(this.map.lock().await.remove(&key).unwrap_or(LuaValue::Nil))
        });
    }
}

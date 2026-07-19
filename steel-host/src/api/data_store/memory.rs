use std::{collections::HashMap, sync::Arc};

use mlua::prelude::*;
use steel_utils::locks::AsyncMutex;

use crate::api::data_store::{DataStore, key::LuaKey};

#[derive(Debug, Clone)]
pub struct MemoryStore {
    map: Arc<AsyncMutex<HashMap<LuaKey, LuaValue>>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }
}

impl DataStore for MemoryStore {
    async fn set_async(&self, key: LuaKey, value: LuaValue) -> LuaResult<()> {
        self.map.lock().await.insert(key, value);
        Ok(())
    }

    async fn update_async(&self, key: LuaKey, update: LuaFunction) -> LuaResult<()> {
        let mut map = self.map.lock().await;
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
    }

    async fn get_async(&self, key: LuaKey) -> LuaResult<LuaValue> {
        let value = self
            .map
            .lock()
            .await
            .get(&key)
            .cloned()
            .unwrap_or(LuaValue::Nil);
        Ok(value)
    }

    async fn remove_async(&self, key: LuaKey) -> LuaResult<LuaValue> {
        let value = self.map.lock().await.remove(&key).unwrap_or(LuaValue::Nil);
        Ok(value)
    }
}

crate::impl_store_userdata!(MemoryStore);

#[cfg(test)]
mod tests {
    use mlua::prelude::*;

    use crate::api::data_store::MemoryStore;

    #[tokio::test]
    async fn test_set_get_async() {
        let lua = Lua::new();
        let globals = lua.globals();

        let store = MemoryStore::new();
        globals
            .set("store", store.clone())
            .expect("failed to set global");

        lua.load(
            r#"
                store:SetAsync("id", 7)
                value = store:GetAsync("id")
        "#,
        )
        .exec_async()
        .await
        .unwrap();

        assert!(matches!(
            globals.get::<LuaValue>("value"),
            Ok(LuaValue::Integer(7))
        ));
    }

    #[tokio::test]
    async fn test_get_async_missing_returns_nil() {
        let lua = Lua::new();
        let globals = lua.globals();

        let store = MemoryStore::new();
        globals
            .set("store", store.clone())
            .expect("failed to set global");

        lua.load(
            r#"
                missing = store:GetAsync("missing")
        "#,
        )
        .exec_async()
        .await
        .unwrap();

        assert!(matches!(
            globals.get::<LuaValue>("missing"),
            Ok(LuaValue::Nil)
        ));
    }

    #[tokio::test]
    async fn test_update_async() {
        let lua = Lua::new();
        let globals = lua.globals();

        let store = MemoryStore::new();
        globals
            .set("store", store.clone())
            .expect("failed to set global");

        lua.load(
            r#"
                store:SetAsync("id", 7)

                store:UpdateAsync("id", function(current)
                    assert(current == 7)
                    return current + 5
                end)

                updated = store:GetAsync("id")
        "#,
        )
        .exec_async()
        .await
        .unwrap();

        assert!(matches!(
            globals.get::<LuaValue>("updated"),
            Ok(LuaValue::Integer(12))
        ));
    }

    #[tokio::test]
    async fn test_remove_async() {
        let lua = Lua::new();
        let globals = lua.globals();

        let store = MemoryStore::new();
        globals
            .set("store", store.clone())
            .expect("failed to set global");

        lua.load(
            r#"
                store:SetAsync("id", 7)

                removed = store:RemoveAsync("id")
                after_remove = store:GetAsync("id")
        "#,
        )
        .exec_async()
        .await
        .unwrap();

        assert!(matches!(
            globals.get::<LuaValue>("removed"),
            Ok(LuaValue::Integer(7))
        ));
        assert!(matches!(
            globals.get::<LuaValue>("after_remove"),
            Ok(LuaValue::Nil)
        ));
    }
}

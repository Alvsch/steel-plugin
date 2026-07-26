use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

use heed::{Env, EnvOpenOptions};
use mlua::{
    Error as LuaError, Function as LuaFunction, Lua, Result as LuaResult, Table as LuaTable,
    Value as LuaValue,
};
use serde::{Deserialize, Serialize};

use crate::{
    api::{DataStore, data_store::lmdb::worker::LmdbClient},
    impl_store_userdata,
};

mod worker;

const MAX_UPDATE_RETRIES: usize = 32;

pub fn open_lmdb_env(
    path: impl AsRef<Path>,
    map_size_bytes: usize,
    max_dbs: u32,
) -> LuaResult<Env> {
    unsafe {
        EnvOpenOptions::new()
            .map_size(map_size_bytes)
            .max_dbs(max_dbs)
            .open(path)
            .map_err(LuaError::external)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[repr(u8)]
pub enum LuaValueRepr {
    Null,
    Bool(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Array(Vec<LuaValueRepr>),
    Dictionary(BTreeMap<String, LuaValueRepr>),
}

pub struct LmdbStore {
    worker: LmdbClient,
}

impl LmdbStore {
    pub fn open(env: Env, table_name: &str) -> LuaResult<Self> {
        let mut txn = env.write_txn().map_err(LuaError::external)?;

        let db = env
            .create_database(&mut txn, Some(table_name))
            .map_err(LuaError::external)?;

        txn.commit().map_err(LuaError::external)?;

        Ok(Self {
            worker: LmdbClient::new(env, db),
        })
    }

    fn table_to_repr(table: &LuaTable, visited: &mut HashSet<usize>) -> LuaResult<LuaValueRepr> {
        let ptr = table.to_pointer() as usize;

        if !visited.insert(ptr) {
            return Err(LuaError::RuntimeError(
                "cyclic tables cannot be stored".into(),
            ));
        }

        let len = table.raw_len();

        let mut is_array = true;
        let mut is_dict = true;

        for pair in table.clone().pairs::<LuaValue, LuaValue>() {
            let (key, _) = pair?;

            match key {
                LuaValue::Integer(i) if i >= 1 && (i as usize) <= len => {
                    is_dict = false;
                }

                LuaValue::String(_) => {
                    is_array = false;
                }

                _ => {
                    return Err(LuaError::RuntimeError(
                        "table keys must be strings or sequential integers".into(),
                    ));
                }
            }
        }

        let result = if len == 0 && is_array && is_dict {
            LuaValueRepr::Array(Vec::new())
        } else if is_array {
            let mut values = Vec::with_capacity(len);

            for i in 1..=len {
                values.push(Self::value_to_repr(&table.raw_get(i as i64)?, visited)?);
            }

            LuaValueRepr::Array(values)
        } else if is_dict {
            let mut map = BTreeMap::new();

            for pair in table.clone().pairs::<String, LuaValue>() {
                let (key, value) = pair?;

                map.insert(key, Self::value_to_repr(&value, visited)?);
            }

            LuaValueRepr::Dictionary(map)
        } else {
            return Err(LuaError::RuntimeError(
                "mixed array/dictionary tables are unsupported".into(),
            ));
        };

        visited.remove(&ptr);

        Ok(result)
    }

    fn value_to_repr(value: &LuaValue, visited: &mut HashSet<usize>) -> LuaResult<LuaValueRepr> {
        Ok(match value {
            LuaValue::Nil => LuaValueRepr::Null,
            LuaValue::Boolean(v) => LuaValueRepr::Bool(*v),
            LuaValue::Integer(v) => LuaValueRepr::Integer(*v),
            LuaValue::Number(v) => LuaValueRepr::Number(*v),
            LuaValue::String(v) => {
                let s = v
                    .to_str()
                    .map_err(|_| LuaError::RuntimeError("stored strings must be UTF-8".into()))?;

                LuaValueRepr::String(s.to_string())
            }
            LuaValue::Table(t) => Self::table_to_repr(t, visited)?,
            other => {
                return Err(LuaError::RuntimeError(format!(
                    "cannot store lua value {:?}",
                    other.type_name()
                )));
            }
        })
    }

    fn repr_to_value(lua: &Lua, repr: &LuaValueRepr) -> LuaResult<LuaValue> {
        Ok(match repr {
            LuaValueRepr::Null => LuaValue::Nil,
            LuaValueRepr::Bool(v) => LuaValue::Boolean(*v),
            LuaValueRepr::Integer(v) => LuaValue::Integer(*v),
            LuaValueRepr::Number(v) => LuaValue::Number(*v),
            LuaValueRepr::String(v) => LuaValue::String(lua.create_string(v)?),
            LuaValueRepr::Array(values) => {
                let table = lua.create_table()?;

                for (i, value) in values.iter().enumerate() {
                    table.raw_set((i + 1) as i64, Self::repr_to_value(lua, value)?)?;
                }

                LuaValue::Table(table)
            }
            LuaValueRepr::Dictionary(values) => {
                let table = lua.create_table()?;

                for (key, value) in values {
                    table.raw_set(key.as_str(), Self::repr_to_value(lua, value)?)?;
                }
                LuaValue::Table(table)
            }
        })
    }
}

impl DataStore for LmdbStore {
    async fn set_async(&self, key: String, value: LuaValue) -> LuaResult<()> {
        let value = Self::value_to_repr(&value, &mut HashSet::new())?;

        self.worker.put(key, value).await
    }

    async fn get_async(&self, lua: &Lua, key: String) -> LuaResult<LuaValue> {
        let raw = self.worker.get(key).await?;

        match raw {
            Some(bytes) => Self::repr_to_value(lua, &bytes),
            None => Ok(LuaValue::Nil),
        }
    }

    async fn update_async(&self, lua: &Lua, key: String, update: LuaFunction) -> LuaResult<()> {
        for _ in 0..MAX_UPDATE_RETRIES {
            let old_bytes = self.worker.get(key.clone()).await?;

            let old_value = match &old_bytes {
                Some(bytes) => Self::repr_to_value(lua, bytes)?,
                None => LuaValue::Nil,
            };

            let new_value: LuaValue = update.call_async(old_value).await?;

            let new_bytes = match new_value {
                LuaValue::Nil => None,
                value => Some(Self::value_to_repr(&value, &mut HashSet::new())?),
            };

            if self
                .worker
                .compare_and_swap(key.clone(), old_bytes, new_bytes)
                .await?
            {
                return Ok(());
            }
        }

        Err(LuaError::RuntimeError(
            "update failed due to contention".into(),
        ))
    }

    async fn remove_async(&self, lua: &Lua, key: String) -> LuaResult<LuaValue> {
        let raw = self.worker.remove(key).await?;

        match raw {
            Some(bytes) => Self::repr_to_value(lua, &bytes),
            None => Ok(LuaValue::Nil),
        }
    }
}

impl_store_userdata!(LmdbStore);

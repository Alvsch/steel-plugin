use std::hash::{Hash, Hasher};

use mlua::prelude::*;

#[derive(Clone, Debug)]
pub struct LuaKey(LuaValue);

impl PartialEq for LuaKey {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (LuaValue::Nil, LuaValue::Nil) => true,
            (LuaValue::Boolean(a), LuaValue::Boolean(b)) => a == b,
            (LuaValue::Integer(a), LuaValue::Integer(b)) => a == b,
            (LuaValue::Number(a), LuaValue::Number(b)) => a.to_bits() == b.to_bits(),
            (LuaValue::String(a), LuaValue::String(b)) => a == b,
            // reference equality for tables, functions, etc.
            (LuaValue::Table(a), LuaValue::Table(b)) => a == b,
            (LuaValue::Function(a), LuaValue::Function(b)) => a == b,
            (LuaValue::UserData(a), LuaValue::UserData(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for LuaKey {}

impl Hash for LuaKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.0 {
            LuaValue::Nil => 0u8.hash(state),
            LuaValue::Boolean(b) => {
                1u8.hash(state);
                b.hash(state);
            }
            LuaValue::Integer(i) => {
                2u8.hash(state);
                i.hash(state);
            }
            LuaValue::Number(f) => {
                3u8.hash(state);
                f.to_bits().hash(state);
            }
            LuaValue::String(s) => {
                4u8.hash(state);
                s.as_bytes().hash(state);
            }
            // pointer-based hash for reference types
            LuaValue::Table(t) => {
                5u8.hash(state);
                t.to_pointer().hash(state);
            }
            LuaValue::Function(f) => {
                6u8.hash(state);
                f.to_pointer().hash(state);
            }
            LuaValue::UserData(u) => {
                7u8.hash(state);
                u.to_pointer().hash(state);
            }
            _ => 8u8.hash(state),
        }
    }
}

impl FromLua for LuaKey {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        Ok(LuaKey(value))
    }
}

impl IntoLua for LuaKey {
    fn into_lua(self, _lua: &Lua) -> LuaResult<LuaValue> {
        Ok(self.0)
    }
}

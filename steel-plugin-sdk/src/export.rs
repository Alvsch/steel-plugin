use std::borrow::Cow;

use serde::{Deserialize, Serialize};

pub use inventory::{iter, submit};
use steel_plugin_core::TopicId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExportedKind {
    Rpc { export_name: Cow<'static, str> },
    Event { topic_id: TopicId, priority: i8 },
    Command,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportedId {
    pub kind: ExportedKind,
    pub id: u32,
}

impl From<Exported> for ExportedId {
    fn from(value: Exported) -> Self {
        Self {
            kind: value.kind,
            id: value.func as usize as u32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Exported {
    pub kind: ExportedKind,
    pub func: fn(u64) -> u64,
}

inventory::collect!(Exported);

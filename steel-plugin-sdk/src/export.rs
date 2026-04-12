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
        match value {
            Exported::Rpc { export_name, func } => Self {
                kind: ExportedKind::Rpc { export_name },
                id: func as usize as u32,
            },
            Exported::Event {
                topic_id,
                priority,
                func,
            } => Self {
                kind: ExportedKind::Event { topic_id, priority },
                id: func as usize as u32,
            },
            Exported::Command { func } => Self {
                kind: ExportedKind::Command,
                id: func as usize as u32,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Exported {
    Rpc {
        export_name: Cow<'static, str>,
        func: fn(u64) -> u64,
    },
    Event {
        topic_id: TopicId,
        priority: i8,
        func: fn(u64),
    },
    Command {
        func: fn(u64) -> u64,
    },
}

inventory::collect!(Exported);

use std::mem::forget;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EventResult {
    pub modified: Option<(u32, u32)>,
}

impl EventResult {
    #[must_use]
    pub fn modified(mut data: Vec<u8>) -> Self {
        data.insert(0, 0); // cancelled false
        let ptr = data.as_ptr() as u32;
        let len = data.len() as u32;
        forget(data);
        Self {
            modified: Some((ptr, len)),
        }
    }

    #[must_use]
    pub fn cancelled() -> Self {
        let boxed = Box::new([true]);
        let ptr = boxed.as_ptr() as u32;
        forget(boxed);
        Self {
            modified: Some((ptr, 1)),
        }
    }

    #[must_use]
    pub const fn pack(&self) -> u64 {
        let Some((ptr, len)) = self.modified else {
            return 0;
        };

        ((ptr as u64) << 32) | len as u64
    }

    #[must_use]
    pub const fn unpack(value: u64) -> Self {
        if value == 0 {
            return Self { modified: None };
        }
        let ptr = (value >> 32) as u32;
        let len = (value & u32::MAX as u64) as u32;
        Self {
            modified: Some((ptr, len)),
        }
    }
}

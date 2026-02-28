use std::{
    mem::forget,
    num::{NonZero, NonZeroU64},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EventResult {
    modified: Option<NonZeroU64>,
}

impl EventResult {
    #[must_use]
    pub fn new(ptr: u32, len: u32) -> Self {
        Self {
            modified: NonZero::new(((u64::from(ptr)) << 32) | u64::from(len)),
        }
    }

    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self {
            modified: NonZeroU64::new(value),
        }
    }
    
    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.modified.map_or(0, NonZeroU64::get)
    }

    #[must_use]
    pub fn unpack(&self) -> Option<(u32, u32)> {
        let value = self.modified?.get();
        let ptr = (value >> 32) as u32;
        let len = (value & u64::from(u32::MAX)) as u32;
        Some((ptr, len))
    }

    #[must_use]
    pub fn modified(mut data: Vec<u8>) -> Self {
        data.insert(0, 0); // cancelled false
        let ptr = data.as_ptr() as u32;
        let len = data.len() as u32;
        forget(data);
        Self::new(ptr, len)
    }

    #[must_use]
    pub fn cancelled() -> Self {
        let boxed = Box::new([true]);
        let ptr = boxed.as_ptr() as u32;
        forget(boxed);
        Self::new(ptr, 1)
    }
}

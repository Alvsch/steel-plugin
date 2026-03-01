use crate::event::Event;
use rmp_serde::to_vec;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::{mem::forget, num::NonZeroU64};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EventResult<T: Event> {
    modified: Option<NonZeroU64>,
    _marker: PhantomData<T>,
}

impl<T: Event> EventResult<T> {
    #[must_use]
    pub fn new(ptr: u32, len: u32) -> Self {
        Self::from_u64(((u64::from(ptr)) << 32) | u64::from(len))
    }

    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self {
            modified: NonZeroU64::new(value),
            _marker: PhantomData,
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
    pub fn modified(event: &T) -> Self {
        let data = to_vec(event).unwrap();
        let ptr = data.as_ptr() as u32;
        let len = data.len() as u32;
        forget(data);
        Self::new(ptr, len)
    }
}

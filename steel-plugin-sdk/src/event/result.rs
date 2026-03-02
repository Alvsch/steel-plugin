use crate::event::Event;
use crate::utils::fat::FatPtr;
use rmp_serde::to_vec;
use std::marker::PhantomData;
use std::mem::forget;

#[derive(Debug, Default)]
pub struct EventResult<T: Event> {
    pub fat: Option<FatPtr>,
    _marker: PhantomData<T>,
}

impl<T: Event> EventResult<T> {
    #[must_use]
    pub const fn new(fat: Option<FatPtr>) -> Self {
        Self {
            fat,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn modified(event: &T) -> Self {
        let data = to_vec(event).unwrap();
        let ptr = data.as_ptr() as u32;
        let len = data.len() as u32;
        forget(data);
        Self::new(FatPtr::new(ptr, len))
    }
}

impl<T: Event> From<u64> for EventResult<T> {
    fn from(value: u64) -> Self {
        Self {
            fat: FatPtr::unpack(value),
            _marker: PhantomData,
        }
    }
}

use std::mem::forget;

#[derive(Debug, Default)]
pub struct EventResult {
    pub modified: Option<(u32, u32)>,
}

impl EventResult {
    #[must_use]
    pub const fn modified(ptr: u32, len: u32) -> Self {
        Self {
            modified: Some((ptr, len)),
        }
    }

    #[must_use]
    pub fn cancelled(cancelled: bool) -> Self {
        let boxed = Box::new([cancelled]);
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

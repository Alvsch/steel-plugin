use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

/// A fat pointer consisting of a 32-bit address and a 32-bit length, packed
/// into a single `u64`.
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FatPtr(NonZeroU64);

impl FatPtr {
    /// Creates a new `FatPtr` from a pointer and length.
    ///
    /// Returns `None` only if both `ptr` and `len` are zero.
    #[must_use]
    pub const fn new(ptr: u32, len: u32) -> Option<Self> {
        match NonZeroU64::new(((ptr as u64) << 32) | len as u64) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    /// Returns the pointer.
    #[must_use]
    pub const fn ptr(self) -> u32 {
        (self.0.get() >> 32) as u32
    }

    /// Returns the length.
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(self) -> u32 {
        self.0.get() as u32
    }

    /// Packs this `FatPtr` into a `u64`.
    #[must_use]
    pub const fn pack(self) -> u64 {
        self.0.get()
    }

    /// Unpacks a `u64` into a `FatPtr`.
    ///
    /// Returns `None` if `packed` is zero.
    #[must_use]
    pub const fn unpack(packed: u64) -> Option<Self> {
        match NonZeroU64::new(packed) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }
}

impl From<FatPtr> for u64 {
    fn from(fat: FatPtr) -> u64 {
        fat.pack()
    }
}

impl TryFrom<u64> for FatPtr {
    type Error = ();

    fn try_from(packed: u64) -> Result<Self, Self::Error> {
        Self::unpack(packed).ok_or(())
    }
}

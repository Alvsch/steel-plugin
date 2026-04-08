use std::{marker::PhantomData, slice};

use serde::{Deserialize, Serialize};
use slotmap::{KeyData, new_key_type};

use crate::{
    host,
    objects::{batch::BatchBuilder, query::QuerySet},
    utils::fat::FatPtr,
};

pub mod batch;
pub mod player;
pub mod query;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[expect(missing_docs, reason = "variant names are self-explanatory")]
pub enum GameType {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

new_key_type! {
    pub struct HandleKey;
}

impl HandleKey {
    /// Converts this key into a plain integer for crossing the WASM ABI.
    #[must_use]
    pub fn as_ffi(self) -> u64 {
        self.0.as_ffi()
    }

    /// Reconstructs a key from its ABI integer representation.
    #[must_use]
    pub const fn from_ffi(value: u64) -> Self {
        Self(KeyData::from_ffi(value))
    }
}

/// Marker trait for entity types.
///
/// Each implementor defines the wire representations for its queries and
/// commands. These enums are what actually cross the WASM boundary.
pub trait Entity: 'static {
    type WireQuery: Serialize;
    type WireCommand: Serialize;
}

/// A typed handle to a specific entity instance.
#[derive(Debug, Serialize, Deserialize)]
#[expect(
    clippy::unsafe_derive_deserialize,
    reason = "safety does not depend on any invariants"
)]
pub struct Handle<E: Entity> {
    key: HandleKey,
    _marker: PhantomData<E>,
}

impl<E: Entity> Handle<E> {
    /// Constructs a handle from a raw entity key.
    #[must_use]
    pub const fn from_raw(key: HandleKey) -> Self {
        Self {
            key,
            _marker: PhantomData,
        }
    }

    /// Returns the raw opaque entity key.
    #[must_use]
    pub const fn key(&self) -> HandleKey {
        self.key
    }

    /// Fetches a set of fields from the host in a single round-trip.
    ///
    /// `Q` is a tuple of query marker types. The return type mirrors
    /// the tuple with each marker replaced by its concrete output type.
    #[must_use]
    pub fn fetch<Q: QuerySet<E>>(&self) -> Q::Output {
        let wire_queries = Q::to_wire();
        let query_payload =
            rmp_serde::to_vec(&wire_queries).expect("failed to serialize fetch queries");

        let fat_ptr = unsafe {
            FatPtr::unpack(host::object_fetch(
                self.key.as_ffi(),
                query_payload.as_ptr() as u32,
                query_payload.len() as u32,
            ))
        }
        .expect("host returned a null pointer");

        let response =
            unsafe { slice::from_raw_parts(fat_ptr.ptr() as *const u8, fat_ptr.len() as usize) };

        Q::from_response(response)
    }

    /// Creates a [`BatchBuilder`] for sending commands to this entity.
    ///
    /// Commands are dispatched when [`.send()`](BatchBuilder::send) is
    /// called on the returned builder.
    pub const fn batch(&self) -> BatchBuilder<E> {
        BatchBuilder::new(self.key)
    }
}

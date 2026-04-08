use std::marker::PhantomData;

use crate::{
    host,
    objects::{Entity, HandleKey},
};

/// A builder for batching commands to an entity.
///
/// Commands are collected locally and dispatched in a single host call when
/// [`BatchBuilder::send`] is called.
#[must_use = "BatchBuilder does nothing unless `.send()` is called"]
pub struct BatchBuilder<E: Entity> {
    key: HandleKey,
    commands: Vec<E::WireCommand>,
    _marker: PhantomData<E>,
}

impl<E: Entity> BatchBuilder<E> {
    pub(crate) const fn new(key: HandleKey) -> Self {
        Self {
            key,
            commands: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Pushes a raw wire command onto the batch.
    pub fn push(mut self, cmd: E::WireCommand) -> Self {
        self.commands.push(cmd);
        self
    }

    /// Dispatches all accumulated commands in a single host call.
    pub fn send(self) {
        let payload =
            rmp_serde::to_vec_named(&self.commands).expect("failed to serialize batch commands");

        unsafe {
            host::object_batch_dispatch(
                self.key.as_ffi(),
                payload.as_ptr() as u32,
                payload.len() as u32,
            );
        }
    }
}

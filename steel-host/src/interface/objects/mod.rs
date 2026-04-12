use std::sync::Arc;

use rmp::encode::write_array_len;
use slotmap::SlotMap;
use steel_plugin_sdk::objects::{Entity, HandleKey};

mod player;
pub use player::make_player_handler;

type FetchHandler = dyn Fn(&[u8]) -> Result<Vec<u8>, String> + Send + Sync;
type BatchHandler = dyn Fn(&[u8]) -> Result<(), String> + Send + Sync;

#[derive(Clone)]
pub struct ObjectHandler {
    fetch: Arc<FetchHandler>,
    batch_dispatch: Arc<BatchHandler>,
}

impl ObjectHandler {
    #[must_use]
    pub fn from_fns<F, B>(fetch: F, batch_dispatch: B) -> Self
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, String> + Send + Sync + 'static,
        B: Fn(&[u8]) -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            fetch: Arc::new(fetch),
            batch_dispatch: Arc::new(batch_dispatch),
        }
    }

    #[must_use]
    pub fn make<E, F, B>(handle_query: F, handle_command: B) -> Self
    where
        E: Entity,
        F: Fn(&mut rmp_serde::Serializer<&mut Vec<u8>>, E::WireQuery) -> Result<(), String>
            + Send
            + Sync
            + 'static,
        B: Fn(E::WireCommand) -> Result<(), String> + Send + Sync + 'static,
    {
        Self::from_fns(
            move |payload| {
                let queries: Vec<E::WireQuery> = rmp_serde::from_slice(payload)
                    .map_err(|err| format!("failed to decode queries: {err}"))?;

                let query_count = u32::try_from(queries.len())
                    .map_err(|_| "too many queries in a single fetch".to_string())?;

                let mut payload = Vec::new();
                write_array_len(&mut payload, query_count)
                    .map_err(|err| format!("failed to encode response array header: {err}"))?;

                let mut serializer = rmp_serde::Serializer::new(&mut payload);
                for query in queries {
                    (handle_query)(&mut serializer, query).map_err(|err| {
                        format!("failed to serialize fetch response value: {err}")
                    })?;
                }
                Ok(payload)
            },
            move |payload| {
                let commands: Vec<E::WireCommand> = rmp_serde::from_slice(payload)
                    .map_err(|err| format!("failed to decode player commands: {err}"))?;

                for command in commands {
                    (handle_command)(command)?;
                }
                Ok(())
            },
        )
    }
}

pub enum FetchOutcome {
    MissingKey,
    Response(Vec<u8>),
    HandlerError(String),
}

pub enum BatchDispatchOutcome {
    Dispatched,
    MissingKey,
    HandlerError(String),
}

pub struct ObjectRegistry {
    handlers: SlotMap<HandleKey, ObjectHandler>,
}

impl Default for ObjectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: SlotMap::with_key(),
        }
    }

    pub fn register(&mut self, handler: ObjectHandler) -> HandleKey {
        self.handlers.insert(handler)
    }

    pub fn unregister(&mut self, key: HandleKey) -> Option<ObjectHandler> {
        self.handlers.remove(key)
    }

    #[must_use]
    pub fn contains(&self, key: HandleKey) -> bool {
        self.handlers.contains_key(key)
    }

    #[must_use]
    pub fn fetch(&self, key: HandleKey, payload: &[u8]) -> FetchOutcome {
        let Some(handler) = self.handlers.get(key) else {
            return FetchOutcome::MissingKey;
        };

        match (handler.fetch)(payload) {
            Ok(response) => FetchOutcome::Response(response),
            Err(err) => FetchOutcome::HandlerError(err),
        }
    }

    #[must_use]
    pub fn batch_dispatch(&self, key: HandleKey, payload: &[u8]) -> BatchDispatchOutcome {
        let Some(handler) = self.handlers.get(key) else {
            return BatchDispatchOutcome::MissingKey;
        };

        match (handler.batch_dispatch)(payload) {
            Ok(()) => BatchDispatchOutcome::Dispatched,
            Err(err) => BatchDispatchOutcome::HandlerError(err),
        }
    }
}

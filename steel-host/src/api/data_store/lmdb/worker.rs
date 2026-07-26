use std::thread;

use heed::{
    Database, Env,
    types::{SerdeRmp, Str},
};
use mlua::{Error as LuaError, Result as LuaResult};
use tokio::sync::{mpsc, oneshot};

use crate::api::data_store::lmdb::LuaValueRepr;

pub enum LmdbRequest {
    Get {
        key: String,
        response: oneshot::Sender<LuaResult<Option<LuaValueRepr>>>,
    },

    Put {
        key: String,
        value: LuaValueRepr,
        response: oneshot::Sender<LuaResult<()>>,
    },

    Remove {
        key: String,
        response: oneshot::Sender<LuaResult<Option<LuaValueRepr>>>,
    },

    CompareAndSwap {
        key: String,
        expected: Option<LuaValueRepr>,
        value: Option<LuaValueRepr>,
        response: oneshot::Sender<LuaResult<bool>>,
    },
}

#[derive(Clone)]
pub struct LmdbClient {
    sender: mpsc::Sender<LmdbRequest>,
}

impl LmdbClient {
    pub fn new(env: Env, db: Database<Str, SerdeRmp<LuaValueRepr>>) -> Self {
        let (sender, receiver) = mpsc::channel(256);

        thread::spawn(move || {
            worker_loop(env, db, receiver);
        });

        Self { sender }
    }

    pub async fn get(&self, key: String) -> LuaResult<Option<LuaValueRepr>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(LmdbRequest::Get { key, response: tx })
            .await
            .map_err(|_| LuaError::RuntimeError("lmdb worker stopped".into()))?;

        rx.await.map_err(LuaError::external)?
    }

    pub async fn put(&self, key: String, value: LuaValueRepr) -> LuaResult<()> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(LmdbRequest::Put {
                key,
                value,
                response: tx,
            })
            .await
            .map_err(|_| LuaError::RuntimeError("lmdb worker stopped".into()))?;

        rx.await.map_err(LuaError::external)?
    }

    pub async fn remove(&self, key: String) -> LuaResult<Option<LuaValueRepr>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(LmdbRequest::Remove { key, response: tx })
            .await
            .map_err(|_| LuaError::RuntimeError("lmdb worker stopped".into()))?;

        rx.await.map_err(LuaError::external)?
    }

    pub async fn compare_and_swap(
        &self,
        key: String,
        expected: Option<LuaValueRepr>,
        value: Option<LuaValueRepr>,
    ) -> LuaResult<bool> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(LmdbRequest::CompareAndSwap {
                key,
                expected,
                value,
                response: tx,
            })
            .await
            .map_err(|_| LuaError::RuntimeError("lmdb worker stopped".into()))?;

        rx.await.map_err(LuaError::external)?
    }
}

fn worker_loop(
    env: Env,
    db: Database<Str, SerdeRmp<LuaValueRepr>>,
    mut receiver: mpsc::Receiver<LmdbRequest>,
) {
    while let Some(request) = receiver.blocking_recv() {
        match request {
            LmdbRequest::Get { key, response } => {
                let result = (|| {
                    let txn = env.read_txn().map_err(LuaError::external)?;

                    db.get(&txn, &key).map_err(LuaError::external)
                })();

                let _ = response.send(result);
            }

            LmdbRequest::Put {
                key,
                value,
                response,
            } => {
                let result = (|| {
                    let mut txn = env.write_txn().map_err(LuaError::external)?;

                    db.put(&mut txn, &key, &value).map_err(LuaError::external)?;

                    txn.commit().map_err(LuaError::external)?;

                    Ok(())
                })();

                let _ = response.send(result);
            }

            LmdbRequest::Remove { key, response } => {
                let result = (|| {
                    let mut txn = env.write_txn().map_err(LuaError::external)?;

                    let old = db.get(&txn, &key).map_err(LuaError::external)?;

                    db.delete(&mut txn, &key).map_err(LuaError::external)?;

                    txn.commit().map_err(LuaError::external)?;

                    Ok(old)
                })();

                let _ = response.send(result);
            }

            LmdbRequest::CompareAndSwap {
                key,
                expected,
                value,
                response,
            } => {
                let result = (|| {
                    let mut txn = env.write_txn().map_err(LuaError::external)?;

                    let current = db.get(&txn, &key).map_err(LuaError::external)?;

                    if current != expected {
                        return Ok(false);
                    }

                    match value {
                        Some(value) => {
                            db.put(&mut txn, &key, &value).map_err(LuaError::external)?;
                        }

                        None => {
                            db.delete(&mut txn, &key).map_err(LuaError::external)?;
                        }
                    }

                    txn.commit().map_err(LuaError::external)?;

                    Ok(true)
                })();

                let _ = response.send(result);
            }
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkPos(pub i32, pub i32);

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockPos(pub i32, pub i32, pub i32);

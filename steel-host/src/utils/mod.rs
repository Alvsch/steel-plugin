use crate::SCRATCH_SIZE;
use crate::plugin::PluginState;
use crate::plugin::{AllocFunc, DeallocFunc};
use steel_plugin_sdk::utils::fat::FatPtr;
use wasmparser::{Parser, Payload};
use wasmtime::{Instance, Memory, Store};

pub mod discover;
pub mod memory;
pub mod sorting;

pub fn read_custom_section<'a>(
    bytes: &'a [u8],
    name: &str,
) -> wasmparser::Result<Option<&'a [u8]>> {
    for payload in Parser::new(0).parse_all(bytes) {
        match payload? {
            Payload::CustomSection(reader) if reader.name() == name => {
                return Ok(Some(reader.data()));
            }
            _ => {}
        }
    }
    Ok(None)
}

/// Writes `data` into scratch if it fits, otherwise heap-allocates via `alloc`.
/// Returns a `FatPtr` describing the written region.
pub async fn write_scratch(
    store: &mut Store<PluginState>,
    memory: Memory,
    alloc: &AllocFunc,
    scratch: FatPtr,
    data: &[u8],
) -> Result<FatPtr, wasmtime::Error> {
    let len = data.len() as u32;
    let ptr = if len > scratch.len() {
        alloc.call_async(&mut *store, len).await?
    } else {
        scratch.ptr()
    };
    memory.write(&mut *store, ptr as usize, data)?;
    Ok(FatPtr::new(ptr, len).unwrap())
}

/// Frees a `FatPtr` produced by `write_scratch`.
/// No-op for scratch allocations, calls `dealloc` for heap allocations.
pub async fn dealloc_scratch(
    store: &mut Store<PluginState>,
    instance: &Instance,
    data: FatPtr,
) -> Result<(), wasmtime::Error> {
    if data.len() > SCRATCH_SIZE {
        let dealloc: DeallocFunc = instance.get_typed_func(&mut *store, "dealloc")?;
        dealloc.call_async(store, (data.ptr(), data.len())).await?;
    }
    Ok(())
}

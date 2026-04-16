use steel_plugin_sdk::{objects::HandleKey, utils::fat::FatPtr};
use tracing::warn;
use wasmtime::Caller;

use crate::{
    error::PluginContractError,
    interface::objects::{BatchDispatchOutcome, FetchOutcome},
    plugin::PluginState,
};

pub async fn fetch(
    mut caller: Caller<'_, PluginState>,
    entity_key: HandleKey,
    queries_fat: FatPtr,
) -> Result<u64, PluginContractError> {
    let exports = caller.data().exports().clone();
    let query_payload = exports.memory.read_memory(&caller, queries_fat).to_vec();

    let outcome = {
        let host = caller.data().host.clone();
        let objects = host.objects.read();
        objects.fetch(entity_key, &query_payload)
    };

    match outcome {
        FetchOutcome::MissingKey => {
            warn!(
                entity_key = entity_key.as_ffi(),
                "object_fetch called with unknown handle key"
            );
            Ok(0)
        }
        FetchOutcome::HandlerError(err) => {
            warn!(entity_key = entity_key.as_ffi(), error = %err, "object_fetch handler failed");
            Ok(0)
        }
        FetchOutcome::Response(response) => {
            let response_len = u32::try_from(response.len()).map_err(|_| {
                PluginContractError::Other("object_fetch response exceeded u32 length".to_string())
            })?;

            let response_fat = exports.alloc(&mut caller, response_len).await?;
            exports
                .memory
                .write_memory(&mut caller, response_fat.ptr(), &response);
            Ok(response_fat.pack())
        }
    }
}

pub async fn batch_dispatch(
    caller: Caller<'_, PluginState>,
    entity_key: HandleKey,
    commands_fat: FatPtr,
) -> Result<(), PluginContractError> {
    let exports = caller.data().exports().clone();
    let command_payload = exports.memory.read_memory(&caller, commands_fat).to_vec();

    let outcome = {
        let host = caller.data().host.clone();
        let objects = host.objects.read().await;
        objects.batch_dispatch(entity_key, &command_payload)
    };

    match outcome {
        BatchDispatchOutcome::Dispatched => (),
        BatchDispatchOutcome::MissingKey => {
            warn!(
                entity_key = entity_key.as_ffi(),
                "object_batch_dispatch called with unknown handle key"
            );
        }
        BatchDispatchOutcome::HandlerError(err) => {
            warn!(entity_key = entity_key.as_ffi(), error = %err, "object_batch_dispatch handler failed");
        }
    }

    Ok(())
}

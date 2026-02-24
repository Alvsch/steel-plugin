use std::path::PathBuf;

use steel_host::{PluginHostData, PluginLoader};
use wasmtime::{Caller, Config, Linker, OptLevel};
use wasmtime_wasi::p1::wasi_snapshot_preview1::add_to_linker;

#[tokio::main]
async fn main() {
    let mut config = Config::new();
    config.async_support(true);
    config.cranelift_opt_level(OptLevel::Speed);
    config.wasm_multi_memory(false);

    let engine = wasmtime::Engine::new(&config).unwrap();
    let mut linker = Linker::new(&engine);
    add_to_linker(&mut linker, |data: &mut PluginHostData| &mut data.wasi).unwrap();
    linker
        .func_wrap(
            "host",
            "print",
            |mut caller: Caller<PluginHostData>, ptr: u32, len: u32| {
                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let mut buf = vec![0u8; len as usize];
                memory.read(&caller, ptr as usize, &mut buf).unwrap();
                let message = String::from_utf8_lossy(&buf);

                let plugin_name = caller.data().plugin_name.as_str();
                println!("[{plugin_name}] {message}");
            },
        )
        .unwrap();

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let loader = PluginLoader::new(engine, linker, path.join("plugins"));

    loader
        .load_plugin(&path.join("target/wasm32-wasip1/debug/steel_plugin.wasm"))
        .await;
}

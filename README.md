# steel-plugin
A plugin system in WebAssembly

## Host usage

```rust
use std::path::Path;

use steel_host::{PluginHost, discover_plugins};

async fn load_all(plugin_dir: &Path, data_dir: &Path) -> anyhow::Result<()> {
    let discovered = discover_plugins(plugin_dir).await?;
    let host = PluginHost::new(data_dir)?;

    for plugin in discovered {
        host.load_plugin(plugin).await?.enable().await?;
    }

    Ok(())
}
```

Discovery returns plugins in dependency order. The host owns plugin state and exposes lookup and event dispatch through `PluginHost`; consumers should not access Wasmtime stores or linker internals.

## Plugin Stages
- unloaded - host has no knowledge of the plugin.
- discovered - wasm file is found. file metadata is read using wasmparser
- compiled - wasm file is compiled into a plugin component.
- instantiated - wasi context created from plugin intent.
- enabled - on_enable called, registering all event handlers. plugin is alive and running.
- disabled - on_disable called. plugin is ready to be cleaned up and unloaded.
- unload - everything about the plugin is forgotten.

## Plugin Meta
- define dependencies
- restrict RPC method resolution to declared dependencies
- `rpc::call` returns `MethodUnavailable` when a target is unavailable or not a declared dependency
- unknown methods return `UnknownMethod`; successful no-response calls return `Ok(None)`
- `rpc::request` and `rpc::notify` provide typed calls encoded as named MessagePack
- typed RPC values are independently defined by each plugin; incompatible payloads can fail at runtime, but matching semantics are not checked across plugins at compile time
- raw byte handlers and `rpc::call` remain available

Typed exports take an owned request type and return either a serializable
response or `()`. The calling plugin defines its own matching payload types:

```rust
use steel_plugin_sdk::{rpc, rpc_export, serde::{Deserialize, Serialize}};

#[derive(Deserialize)]
struct Ping { name: String }

#[derive(Serialize)]
struct Pong { greeting: String }

#[rpc_export]
fn greet(request: Ping) -> Pong {
    Pong { greeting: format!("hello {}", request.name) }
}

let response: Pong = rpc::request("provider-plugin", "greet", &Ping {
    name: "Ada".to_owned(),
})?;
```

Plugin crates using derive macros must include `serde` with its `derive`
feature in their dependencies. `rpc::notify` is the typed form for exports
returning `()`.

The current component contract and plugin metadata API are both version `0.3.0`.
The host accepts plugins with the same major version and an equal or lower
minor version. Before `1.0.0`, compatible minor-version changes may still break
at runtime.

## Testing

- Unit tests for deterministic logic in core/sdk/host.
- Compile-time macro tests for proc-macro validation and diagnostics.
- Host integration tests that run real wasm plugins.

For the contributor workflow for extending the host/plugin contract, see
[Adding a plugin interface](docs/adding-a-plugin-interface.md).

For the host crate's public API and internal ownership model, see
[`steel-host/README.md`](steel-host/README.md).

For longer-term project direction, see [`ROADMAP.md`](ROADMAP.md).

### Common commands

- `just test` runs fast unit-oriented suites.
- `just test-integration` builds wasm plugins and runs host integration tests.

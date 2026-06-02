# steel-plugin
A plugin system in WebAssembly

## Plugin Stages
- unloaded - host has no knowledge of the plugin.
- discovered - wasm file is found.
- preload - file metadata is read using wasmparser (name/api version/intent etc.)
- precompile - wasm file is compiled into a plugin component.
- load - wasi context created from plugin intent and plugin is loaded.
- enabled - on_enable called, registering all event handlers. plugin is alive and running.
- disabled - on_disable called. plugin is ready to be cleaned up and unloaded.
- unload - everything about the plugin is forgotten.

## Plugin Meta
- define dependencies
- restrict rpc resolve_plugin access to those dependencies
- soft dependencies resolve_plugin return Option<u32>

## Testing

- Unit tests for deterministic logic in core/sdk/host.
- Compile-time macro tests for proc-macro validation and diagnostics.
- Host integration tests that run real wasm plugins.

### Common commands

- `just test` runs fast unit-oriented suites.
- `just test-integration` builds wasm plugins and runs host integration tests.

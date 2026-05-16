# steel-plugin
A plugin system in WebAssembly

## Plugin Stages
- unloaded - host has no knowledge of the plugin.
- discovered - wasm file path, name, api_version, etc. gathered statically.
- loaded - wasm compiled and component initialized.
- enabled - on_enable called. plugin is alive and running.
- disabled - on_disable called. plugin is ready to be cleaned up and unloaded.

## Testing

- Unit tests for deterministic logic in core/sdk/host.
- Compile-time macro tests for proc-macro validation and diagnostics.
- Host integration tests that run real wasm plugins.

### Common commands

- `just test` runs fast unit-oriented suites.
- `just test-integration` builds wasm plugins and runs host integration tests.

# steel-plugin
A plugin system in WebAssembly

## Testing

- Unit tests for deterministic logic in core/sdk/host.
- Compile-time macro tests for proc-macro validation and diagnostics.
- Host integration tests that run real wasm plugins.

### Common commands

- `just test` runs fast unit-oriented suites.
- `just test-macros` runs full macro tests (including compile-fail cases).
- `just test-integration` builds wasm plugins and runs host integration tests.

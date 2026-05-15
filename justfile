build-plugin:
    cargo build -p listening-plugin --target wasm32-wasip2 --profile profiling
    cargo build -p provider-plugin --target wasm32-wasip2 --profile profiling
    cargo build -p consumer-plugin --target wasm32-wasip2 --profile profiling

build: build-plugin
    cargo build -p steel-host

run:
    cargo run --bin steel-host

fmt:
    cargo fmt

clippy:
    cargo clippy --workspace --all-targets

check:
    cargo check

test:
    cargo test -p steel-plugin-core
    cargo test -p steel-plugin-sdk
    cargo test -p steel-plugin-macros --lib
    cargo test -p steel-host --lib

test-integration: build-plugin
    cargo test -p steel-host --tests

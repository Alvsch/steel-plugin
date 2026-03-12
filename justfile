build-plugin:
    cargo build -p listening-plugin --target wasm32-wasip1 --profile profiling
    cargo build -p provider-plugin --target wasm32-wasip1 --profile profiling
    cargo build -p consumer-plugin --target wasm32-wasip1 --profile profiling

build: build-plugin
    cargo build -p steel-host

run:
    cargo run --bin steel-host

fmt:
    cargo fmt

clippy:
    cargo clippy

check:
    cargo check

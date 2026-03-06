build-plugin:
    cargo build -p provider-plugin --target wasm32-wasip1
    cargo build -p consumer-plugin --target wasm32-wasip1

build: build-plugin
    cargo build -p steel-host

run: build
    cargo run --bin steel-host

fmt:
    cargo fmt

clippy:
    cargo clippy

check:
    cargo check

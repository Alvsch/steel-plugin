build-plugin:
    cargo build -p steel-plugin --target wasm32-wasip1

build: build-plugin
    cargo build -p steel-host

fmt:
    cargo fmt

clippy:
    cargo clippy

check:
    cargo check

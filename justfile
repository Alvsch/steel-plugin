build-plugin:
    cargo build -p provider-plugin --target wasm32-wasip1 --release
    cargo build -p consumer-plugin --target wasm32-wasip1 --release

build: build-plugin
    cargo build -p steel-host

run: build-plugin
    cargo run --bin steel-host

fmt:
    cargo fmt

clippy:
    cargo clippy

check:
    cargo check

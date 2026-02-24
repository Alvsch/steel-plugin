build-plugin:
    cargo build -p steel-plugin --target wasm32-wasip1

run: build-plugin
    cargo run --bin steel-host

build: build-plugin
    cargo build --bin steel-host

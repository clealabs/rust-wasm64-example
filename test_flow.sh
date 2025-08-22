cd hello-world-wasm64
cargo build -Z build-std=std,panic_abort --target wasm64-unknown-unknown --release
cd ../host-runner
cargo build --release
cargo run --release
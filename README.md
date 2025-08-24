# Rust WASM64 Example

This is an example of how to print a string from a WASM64 guest module (compiled to target `wasm64-unknown-unknown`) through a host runner (here `wasmtime` is used) using a FFI.

## Build and Run with wasmtime

```bash
./test_flow.sh
```

## Run in the browser

Serve the current directory with any http server, for example:

```
npm install -g http-server
http-server -c-1
```

Then open `http://localhost:8080/index.html` in Chrome or Firefox.

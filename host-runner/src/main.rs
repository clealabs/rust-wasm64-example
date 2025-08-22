use anyhow::{anyhow, Result};
use std::fs::read;
use wasmtime::Caller;
use wasmtime::Extern;
use wasmtime::*;

fn main() -> Result<()> {
    let mut config = Config::new();
    config.wasm_memory64(true);
    let engine = Engine::new(&config)?;

    // load guest wasm
    let wasm = read(
        "../hello-world-wasm64/target/wasm64-unknown-unknown/release/hello_world_wasm64.wasm",
    )?;
    let module = Module::new(&engine, &wasm)?;

    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);

    // provide the host `host_print(ptr: u64, len: u64)` function in module "host"
    linker.func_wrap(
        "host",
        "host_print",
        |mut caller: Caller<'_, ()>, ptr: u64, len: u64| -> Result<()> {
            // Find exported memory
            let memory = match caller.get_export("memory") {
                Some(Extern::Memory(m)) => m,
                _ => return Err(anyhow!("module did not export memory")),
            };

            // allocate a buffer and read from guest memory (ptr/len are u64)
            let mut buf = vec![0u8; len as usize];
            memory
                .read(&caller, ptr as usize, &mut buf)
                .map_err(|e| anyhow!("memory read failed: {:?}", e))?;

            // print the string (safe to use lossy conversion)
            println!("{}", String::from_utf8_lossy(&buf));
            Ok(())
        },
    )?;

    // instantiate and call `run`
    let instance = linker.instantiate(&mut store, &module)?;
    let run = instance.get_typed_func::<(), ()>(&mut store, "run")?;
    run.call(&mut store, ())?;

    Ok(())
}

use anyhow::{Result, anyhow};
use std::fs::read;
use std::io::Write;
use wasmtime::Caller;
use wasmtime::Extern;
use wasmtime::*;

use wasi_common::WasiCtx;
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasi_common::sync::WasiCtxBuilder;

fn main() -> Result<()> {
    let mut config = Config::new();
    config.wasm_memory64(true);
    config.async_support(false); // disable async support so we use the sync wasi_common components
    let engine = Engine::new(&config)?;

    // load guest wasm
    let wasm = read(
        "../hello-world-wasm64/target/wasm64-unknown-unknown/release/hello_world_wasm64.wasm",
    )?;
    let module = Module::new(&engine, &wasm)?;

    // 3) Create in-memory pipes for stdio (these implement WasiFile)
    //    - we clone them: one clone goes into the WasiCtx, the other we keep here
    let stdout_pipe = WritePipe::new_in_memory();
    let stderr_pipe = WritePipe::new_in_memory();

    // 4) Build a WasiCtx and register the pipes as the module's stdio
    let mut wasi_builder = WasiCtxBuilder::new();
    wasi_builder
        .stdin(Box::new(ReadPipe::from(String::new()))) // empty stdin; replace if you want input
        .stdout(Box::new(stdout_pipe.clone()))
        .stderr(Box::new(stderr_pipe.clone()));
    let wasi_ctx = wasi_builder.build();

    let mut store: Store<WasiCtx> = Store::<WasiCtx>::new(&engine, wasi_ctx);
    let mut linker: Linker<WasiCtx> = Linker::<WasiCtx>::new(&engine);

    // // host_print import (same as your guest expects). Caller state is 'WasiCtx',
    // // but we'll write into the shared stdout_pipe clone we have here.
    // // Note: `ptr` and `len` are u64 because this is wasm64 guest.
    // let stdout_clone_for_host = stdout_pipe.clone();
    // let stderr_clone_for_host = stderr_pipe.clone();

    // provide the host `host_print(ptr: u64, len: u64)` function in module "host"
    // linker.func_wrap(
    //     "host",
    //     "host_print",
    //     |mut caller: Caller<'_, ()>, ptr: u64, len: u64| -> Result<()> {
    //         // Find exported memory
    //         let memory = match caller.get_export("memory") {
    //             Some(Extern::Memory(m)) => m,
    //             _ => return Err(anyhow!("module did not export memory")),
    //         };

    //         // allocate a buffer and read from guest memory (ptr/len are u64)
    //         let mut buf = vec![0u8; len as usize];
    //         memory
    //             .read(&caller, ptr as usize, &mut buf)
    //             .map_err(|e| anyhow!("memory read failed: {:?}", e))?;

    //         // print the string (safe to use lossy conversion)
    //         println!("{}", String::from_utf8_lossy(&buf));
    //         Ok(())
    //     },
    // )?;

    // instantiate and call `run`
    let instance = linker.instantiate(&mut store, &module)?;
    let run = instance.get_typed_func::<(), ()>(&mut store, "run")?;
    run.call(&mut store, ())?;

    // 8) drop the store to drop the WasiCtx inside it (so our clone becomes the sole owner)
    drop(store);

    // 9) extract the in-memory stdout/stderr contents and print them on host
    if let Ok(inner) = stdout_pipe.try_into_inner() {
        // inner is a Cursor<Vec<u8>>
        let bytes = inner.into_inner();
        if !bytes.is_empty() {
            eprintln!("--- captured guest stdout (raw bytes) ---");
            // print as UTF-8 lossy to avoid panic on invalid bytes
            println!("{}", String::from_utf8_lossy(&bytes));
        }
    } else {
        eprintln!("warning: couldn't extract stdout pipe; it may still be referenced elsewhere");
    }

    if let Ok(inner) = stderr_pipe.try_into_inner() {
        let bytes = inner.into_inner();
        if !bytes.is_empty() {
            eprintln!("--- captured guest stderr (raw bytes) ---");
            eprintln!("{}", String::from_utf8_lossy(&bytes));
        }
    } else {
        eprintln!("warning: couldn't extract stderr pipe; it may still be referenced elsewhere");
    }

    Ok(())
}

// Guest: imports a host function `host_print(ptr: u64, len: u64)` from module "host"
// and exports `run` which calls it with a pointer/length into the guest linear memory.
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    fn host_print(ptr: u64, len: u64);
}

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    let s = "hello, wasm64-unknown-unknown!";
    let ptr = s.as_ptr() as u64;
    let len = s.len() as u64;

    unsafe {
        host_print(ptr, len);
    }
}

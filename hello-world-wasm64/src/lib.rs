// Guest: imports a host function `host_print(ptr: u64, len: u64)` from module "host"
// and exports `run` which calls it with a pointer/length into the guest linear memory.
use std::panic;

#[link(wasm_import_module = "host")]
unsafe extern "C" {
    fn host_print(ptr: u64, len: u64);
}

/// https://github.com/rustwasm/console_error_panic_hook/blob/master/src/lib.rs
pub fn hook(info: &panic::PanicHookInfo) {
    let msg = info.to_string();
    let ptr = msg.as_ptr() as u64;
    let len = msg.len() as u64;
    unsafe {
        host_print(ptr, len);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    panic::set_hook(Box::new(hook));

    let s = "hello, wasm64-unknown-unknown!";
    let ptr = s.as_ptr() as u64;
    let len = s.len() as u64;

    unsafe {
        host_print(ptr, len);
        panic!("boom");
    }
}

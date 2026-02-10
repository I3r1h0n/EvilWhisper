use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::winuser::{MessageBoxA, MB_OK};

use std::ptr::null_mut;

#[unsafe(no_mangle)]
pub extern "system" fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(
    _hinst_dll: HINSTANCE,
    dw_reason: DWORD,
    _lp_reserved: LPVOID,
) -> BOOL {
    if dw_reason == DLL_PROCESS_ATTACH {
        attach();
    }

    TRUE
}

fn attach() {
    unsafe {
        MessageBoxA(
            null_mut(),
            b"Hello from Rust DLL\0".as_ptr() as *const i8,
            b"Hello from Rust DLL\0".as_ptr() as *const i8,
            MB_OK,
        );
    }
}

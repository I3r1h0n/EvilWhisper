use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::winuser::{MessageBoxA, MB_OK};

use std::ffi::CString;
use std::ptr::null_mut;

use crate::helpers::locate_main_thread;

mod helpers;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(
    _hinst_dll: HINSTANCE,
    dw_reason: DWORD,
    _lp_reserved: LPVOID,
) -> BOOL {
    if dw_reason == DLL_PROCESS_ATTACH {
        start();
    }

    TRUE
}

fn start() {
    unsafe {
        // Send a startup message
        MessageBoxA(
            null_mut(),
            b"Hello from Rust DLL\0".as_ptr() as *const i8,
            b"Hello from Rust DLL\0".as_ptr() as *const i8,
            MB_OK,
        );
    
        // Find a main thread
        let tid = locate_main_thread();

        // Convert tid to string
        let tid_string = format!("Thread ID: {}", tid);
        let c_tid_string = CString::new(tid_string).unwrap();

        // Send finish message with tid
        MessageBoxA(
            null_mut(),
            c_tid_string.as_ptr(),
            b"Thread ID\0".as_ptr() as *const i8,
            MB_OK,
        );
    }
}

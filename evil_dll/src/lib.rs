use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::um::synchapi::Sleep;
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::winuser::{MessageBoxA, MB_OK};

use std::ptr::null_mut;

use crate::helpers::{locate_main_thread, suspend_thread_by_id};

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

        // Suspend main thread
        if !suspend_thread_by_id(tid) {
            return;
        }

        MessageBoxA(
            null_mut(),
            b"Finish\0".as_ptr() as *const i8,
            b"Finish\0".as_ptr() as *const i8,
            MB_OK,
        );

        Sleep(1000000000);
    }
}

use std::{mem, ptr};

use winapi::{
    shared::{
        minwindef::{DWORD, FALSE, FILETIME, LPARAM, TRUE},
        windef::HWND
    },
    um::{
        handleapi::CloseHandle,
        processthreadsapi::{GetCurrentProcessId, GetThreadTimes, OpenThread, SuspendThread},
        tlhelp32::{CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next},
        winnt::{HANDLE, THREAD_QUERY_INFORMATION, THREAD_QUERY_LIMITED_INFORMATION, THREAD_SUSPEND_RESUME},
        winuser::{EnumWindows, GW_OWNER, GetWindow, GetWindowThreadProcessId, IsWindowVisible}
    }
};

#[inline]
fn filetime_to_u64(ft: &FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
}

pub fn suspend_thread_by_id(tid: DWORD) -> bool {
    unsafe {
        let h_thread: HANDLE = OpenThread(THREAD_SUSPEND_RESUME, FALSE as i32, tid);
        if h_thread.is_null() {
            return false;
        }

        let suspend_count: DWORD = SuspendThread(h_thread);
        if suspend_count == u32::MAX {
            CloseHandle(h_thread);
            return false;
        }

        CloseHandle(h_thread);
        true
    }
}

/// Checks earliest creation
fn find_main_thread_id_by_creation_time(pid: DWORD) -> DWORD {
    let mut best_tid: DWORD = 0;
    let mut best_create: u64 = u64::MAX;

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
            return 0;
        }

        let mut te: THREADENTRY32 = mem::zeroed();
        te.dwSize = mem::size_of::<THREADENTRY32>() as u32;

        if Thread32First(snapshot, &mut te) != FALSE {
            loop {
                if te.th32OwnerProcessID == pid {
                    let rights = THREAD_QUERY_INFORMATION | THREAD_QUERY_LIMITED_INFORMATION;
                    let h_thread: HANDLE = OpenThread(rights, FALSE as i32, te.th32ThreadID);
                    if !h_thread.is_null() {
                        let mut create_time: FILETIME = mem::zeroed();
                        let mut exit_time: FILETIME = mem::zeroed();
                        let mut kernel_time: FILETIME = mem::zeroed();
                        let mut user_time: FILETIME = mem::zeroed();

                        if GetThreadTimes(
                            h_thread,
                            &mut create_time,
                            &mut exit_time,
                            &mut kernel_time,
                            &mut user_time,
                        ) != FALSE
                        {
                            let ct = filetime_to_u64(&create_time);
                            if ct < best_create {
                                best_create = ct;
                                best_tid = te.th32ThreadID;
                            }
                        }
                        CloseHandle(h_thread);
                    }
                }

                if Thread32Next(snapshot, &mut te) == FALSE {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }

    best_tid
}

/// Enumerate top-level windows
fn find_main_window_for_pid(pid: DWORD) -> Option<HWND> {
    #[repr(C)]
    struct Ctx {
        pid: DWORD,
        hwnd: HWND,
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
        let ctx_ptr = lparam as *mut Ctx;
        if ctx_ptr.is_null() {
            return TRUE as i32;
        }

        let ctx = unsafe { &mut *ctx_ptr };

        let mut wpid: DWORD = 0;
        // returns thread id; second param receives process id
        unsafe { 
            GetWindowThreadProcessId(hwnd, &mut wpid as *mut DWORD);
            // check top-level, visible window without owner
            let owner = GetWindow(hwnd, GW_OWNER);
            if wpid == ctx.pid && owner.is_null() && IsWindowVisible(hwnd) != FALSE {
                ctx.hwnd = hwnd;
                return FALSE as i32; // stop enumeration
            }
        };
        TRUE as i32 // continue
    }

    let mut boxed = Box::new(Ctx { pid, hwnd: ptr::null_mut() });
    let raw = Box::into_raw(boxed);

    unsafe {
        EnumWindows(Some(enum_windows_proc), raw as LPARAM);
        boxed = Box::from_raw(raw);
        if boxed.hwnd.is_null() {
            None
        } else {
            Some(boxed.hwnd)
        }
    }
}

/// Located process main thread
pub fn locate_main_thread() -> DWORD {
    // Get current process id
    let pid: DWORD = unsafe {
        GetCurrentProcessId()
    };

    unsafe {
        if let Some(main_hwnd) = find_main_window_for_pid(pid) {
            let tid = GetWindowThreadProcessId(main_hwnd, ptr::null_mut());
            if tid != 0 {
                return tid;
            }
        }
    }
    find_main_thread_id_by_creation_time(pid)
}
